use crate::{
    db::{DbPool, PooledConnection},
    processes::{
        parameters::{
            DataResource, DataResourceSchema, DocumentationSource, FeatureCollectionGeoJsonInput,
            Fields, Hectare, Kilometers, RelativeJsonPointer, TableSchemaField, TableSchemaType,
        },
        util::json_input_value,
    },
    util::{md_content, md_heading},
};
use anyhow::{Context, Result};
use approx::AbsDiffEq;
use diesel::{deserialize::QueryableByName, sql_query, sql_types};
use diesel_async::RunQueryDsl;
use indoc::formatdoc;
use ogcapi::{
    processes::Processor,
    types::processes::{
        Execute, ExecuteResult, ExecuteResults, Format, InlineOrRefData, InputValueNoObject,
        JobControlOptions, Output, Process, ProcessSummary, QualifiedInputValue, TransmissionMode,
        description::{DescriptionType, InputDescription, Metadata, OutputDescription},
    },
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use tracing::instrument;
use utoipa::ToSchema;
use wkt::ToWkt;

/// ## Operation Buffer Zones
///
/// Cf. <https://www.ibat-alliance.org/biodiversity-disclosure>
///
mod buffers {
    use super::*;

    pub const OFFICE_IMPACT: Kilometers = Kilometers(5.0);
    pub const AGRICULTURE_IMPACT: Kilometers = Kilometers(10.0);
    pub const MARINE_IMPACT: Kilometers = Kilometers(20.0);
    pub const MINING_IMPACT: Kilometers = Kilometers(50.0);
    pub const OTHER_IMPACT: Kilometers = Kilometers(20.0);
}

#[doc = include_str!("input_site_type_property.md")]
#[derive(Debug, Clone, Copy, Serialize)]
enum SiteSpecification {
    Office,
    Agriculture,
    Marine,
    Mining,
    Other,
}

impl SiteSpecification {
    /// Return the buffer distance in kilometers for the site specification.
    ///
    /// For a buffer around a point, the distance would be the radius of a circle around the point.
    /// For a polygon, the distance would be added around the whole polygon.
    const fn buffer_distance(self) -> Kilometers {
        match self {
            SiteSpecification::Office => buffers::OFFICE_IMPACT,
            SiteSpecification::Agriculture => buffers::AGRICULTURE_IMPACT,
            SiteSpecification::Marine => buffers::MARINE_IMPACT,
            SiteSpecification::Mining => buffers::MINING_IMPACT,
            SiteSpecification::Other => buffers::OTHER_IMPACT,
        }
    }
}

impl FromStr for SiteSpecification {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "office" => Ok(SiteSpecification::Office),
            "agriculture" => Ok(SiteSpecification::Agriculture),
            "marine" => Ok(SiteSpecification::Marine),
            "mining" => Ok(SiteSpecification::Mining),
            "other" => Ok(SiteSpecification::Other),
            other => Err(format!("Unrecognized site specification `{other}`")),
        }
    }
}

/// Biodiversity impact metrics related to biodiversity and ecosystems change (ESRS E4 B5) - scaffold implementation
#[derive(Debug, Clone)]
pub struct BiodiversitySensitiveAreasProcess {
    connection: DbPool,
    natura2000_schema: &'static str,
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BiodiversitySensitiveAreasProcessInputs {
    /// Collection of all sites to be analyzed, including their location and specification (e.g. office building, agricultural field, mine, etc.).
    /// The impact radius will be determined based on the specification of each site (e.g. 5 km for office buildings, 10 km for agricultural fields, etc.).
    pub sites: FeatureCollectionGeoJsonInput,

    /// Name of the property in the input `GeoJSON` features that contains the location information.
    pub location_property: RelativeJsonPointer,

    /// Name of the property in the input `GeoJSON` features that contains the site type information.
    pub site_type_property: RelativeJsonPointer,
}

mod input_keys {
    pub const SITES: &str = "sites";
    pub const LOCATION_PROPERTY: &str = "locationProperty";
    pub const SITE_TYPE_PROPERTY: &str = "siteTypeProperty";
}

#[derive(Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BiodiversitySensitiveAreasProcessOutputs {
    /// Sites in or near biodiversity-sensitive areas
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub biodiversity_sensitive_areas: Option<DataResource<Vec<SiteRow>>>,

    /// Echo of inputs for auditing and traceability
    pub inputs: Option<BiodiversitySensitiveAreasProcessInputs>,

    /// Errors encountered during processing, if any (e.g. invalid geometries, missing properties, etc.)
    pub errors: Option<Vec<String>>,

    /// Data sources and workflow references used for audits and provenance
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub documentation_sources: Option<DataResource<Vec<DocumentationSource>>>,
}

mod output_keys {
    pub const BIODIVERSITY_SENSITIVE_AREAS: &str = "biodiversitySensitiveAreas";
    pub const INPUTS: &str = "inputs";
    pub const ERRORS: &str = "errors";
    pub const DOCUMENTATION_SOURCES: &str = "documentationSources";
}

#[async_trait::async_trait]
impl Processor for BiodiversitySensitiveAreasProcess {
    fn id(&self) -> &'static str {
        "biodiversity-sensitive-areas"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    #[allow(
        clippy::too_many_lines,
        reason = "This function is verbose due to the detailed process description and schema generation."
    )]
    fn process(&self) -> Result<Process> {
        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;

        let mut generator = settings.into_generator();

        Ok(Process {
            summary: ProcessSummary {
                id: self.id().into(),
                version: self.version().into(),
                job_control_options: vec![
                    JobControlOptions::SyncExecute,
                    JobControlOptions::AsyncExecute,
                ],
                output_transmission: vec![TransmissionMode::Value],
                links: vec![],
            },
            inputs: HashMap::from([
                (
                    input_keys::SITES.to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: Some(md_heading(include_str!("input_sites.md")).to_string()),
                            description: md_content(include_str!("input_sites.md"))
                                .to_string()
                                .into(),
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<FeatureCollectionGeoJsonInput>()
                            .to_value(),
                        ..Default::default()
                    },
                ),
                (
                    input_keys::LOCATION_PROPERTY.to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                md_heading(include_str!("input_location_property.md")).to_string(),
                            ),
                            description: md_content(include_str!("input_location_property.md"))
                                .to_string()
                                .into(),
                            metadata: vec![Metadata {
                                title: Some("GeoJSON Property Pointer".to_string()),
                                role: Some("json-pointer-base".to_string()),
                                href: Some(
                                    "#/inputs/sites/value/features/0/properties".to_string(),
                                ),
                            }],
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<RelativeJsonPointer>()
                            .to_value(),
                        ..Default::default()
                    },
                ),
                (
                    input_keys::SITE_TYPE_PROPERTY.to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                md_heading(include_str!("input_site_type_property.md")).to_string(),
                            ),
                            description: md_content(include_str!("input_site_type_property.md"))
                                .to_string()
                                .into(),
                            metadata: vec![Metadata {
                                title: Some("GeoJSON Property Pointer".to_string()),
                                role: Some("json-pointer-base".to_string()),
                                href: Some(
                                    "#/inputs/sites/value/features/0/properties".to_string(),
                                ),
                            }],
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<RelativeJsonPointer>()
                            .to_value(),
                        ..Default::default()
                    },
                ),
            ]),
            outputs: HashMap::from([
                (
                    output_keys::BIODIVERSITY_SENSITIVE_AREAS.to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                md_heading(include_str!("output_biodiversity_sensitive_areas.md"))
                                    .to_string(),
                            ),
                            description: md_content(include_str!(
                                "output_biodiversity_sensitive_areas.md"
                            ))
                            .to_string()
                            .into(),
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<DataResource<Vec<SiteRow>>>()
                            .to_value(),
                    },
                ),
                (
                    output_keys::INPUTS.to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(md_heading(include_str!("output_inputs.md")).to_string()),
                            description: md_content(include_str!("output_inputs.md"))
                                .to_string()
                                .into(),
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<BiodiversitySensitiveAreasProcessInputs>()
                            .to_value(),
                    },
                ),
                (
                    output_keys::ERRORS.to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(md_heading(include_str!("output_errors.md")).to_string()),
                            description: md_content(include_str!("output_errors.md"))
                                .to_string()
                                .into(),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<Vec<String>>().to_value(),
                    },
                ),
                (
                    output_keys::DOCUMENTATION_SOURCES.to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                md_heading(include_str!("output_documentation_sources.md"))
                                    .to_string(),
                            ),
                            description: md_content(include_str!(
                                "output_documentation_sources.md"
                            ))
                            .to_string()
                            .into(),
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<DataResource<Vec<DocumentationSource>>>()
                            .to_value(),
                    },
                ),
            ]),
        })
    }

    async fn execute(&self, mut execute: Execute) -> Result<ExecuteResults> {
        let value = serde_json::to_value(execute.inputs)?;
        let inputs: BiodiversitySensitiveAreasProcessInputs = serde_json::from_value(value)?;

        // If no outputs were requested, default to all outputs
        if execute.outputs.is_empty() {
            for key in [
                output_keys::BIODIVERSITY_SENSITIVE_AREAS,
                output_keys::ERRORS,
                output_keys::INPUTS,
                output_keys::DOCUMENTATION_SOURCES,
            ] {
                execute.outputs.insert(
                    key.to_string(),
                    Output {
                        format: None,
                        transmission_mode: TransmissionMode::Value,
                    },
                );
            }
        }

        let mut outputs = BiodiversitySensitiveAreasProcessOutputs {
            biodiversity_sensitive_areas: None,
            errors: None,
            inputs: execute
                .outputs
                .contains_key(output_keys::INPUTS)
                .then(|| inputs.clone()),
            documentation_sources: execute.outputs.contains_key(output_keys::DOCUMENTATION_SOURCES).then(
                || {
                    vec![DocumentationSource {
                        data: "Natura 2000 sites (state of 2024).".to_string(), // TODO: get state from dataset metadata
                        documentation_source: "https://environment.ec.europa.eu/topics/nature-and-biodiversity/natura-2000_en".to_string(),
                    }].into()
                },
            ),
        };

        if execute
            .outputs
            .contains_key(output_keys::BIODIVERSITY_SENSITIVE_AREAS)
            || execute.outputs.contains_key(output_keys::ERRORS)
        {
            let (site_table, errors) = compute_biodiversity_sensitive_areas(
                self.connection().await?,
                self.natura2000_schema,
                inputs.sites,
                inputs.location_property.as_ref(),
                inputs.site_type_property.as_ref(),
            )
            .await?;

            outputs.biodiversity_sensitive_areas = execute
                .outputs
                .contains_key(output_keys::BIODIVERSITY_SENSITIVE_AREAS)
                .then_some(site_table.into());

            outputs.errors = execute
                .outputs
                .contains_key(output_keys::ERRORS)
                .then_some(errors);
        }

        Ok(outputs.into())
    }
}

impl BiodiversitySensitiveAreasProcess {
    pub async fn new(connection: DbPool, natura2000_schema: &'static str) -> Result<Self> {
        let this = Self {
            connection,
            natura2000_schema,
        };

        let mut conn = this.connection().await?;
        let table: Natura2000Exists = sql_query(formatdoc! {"
            SELECT EXISTS (
                SELECT 1
                  FROM information_schema.tables
                 WHERE table_schema = '{natura2000_schema}'
                   AND table_name = 'naturasite_polygon'
            ) as exists
        "})
        .get_result(&mut *conn)
        .await
        .context(format!(
            "Failed to check if {natura2000_schema}.naturasite_polygon exists"
        ))?;

        if !table.exists {
            anyhow::bail!("Table {natura2000_schema}.naturasite_polygon does not exist");
        }

        // drop the pooled connection before moving `this` out
        drop(conn);

        Ok(this)
    }

    async fn connection(&self) -> anyhow::Result<PooledConnection<'_>> {
        self.connection
            .get()
            .await
            .context("could not get db connection from pool")
    }
}

#[derive(QueryableByName)]
struct Natura2000Exists {
    #[diesel(sql_type = sql_types::Bool)]
    exists: bool,
}

fn json_format() -> Format {
    Format {
        media_type: Some("application/json".to_string()),
        encoding: None,
        schema: None, // TODO: include JSON schema for outputs
    }
}

impl From<BiodiversitySensitiveAreasProcessOutputs> for ExecuteResults {
    fn from(outputs: BiodiversitySensitiveAreasProcessOutputs) -> Self {
        let mut result = ExecuteResults::default();

        if let Some(biodiversity_sensitive_areas) = outputs.biodiversity_sensitive_areas
            && let Ok(value) = biodiversity_sensitive_areas.to_input_value()
        {
            result.insert(
                output_keys::BIODIVERSITY_SENSITIVE_AREAS.to_string(),
                ExecuteResult {
                    output: Output {
                        format: Some(json_format()),
                        transmission_mode: TransmissionMode::Value,
                    },
                    data: InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
                        value,
                        format: Format {
                            media_type: Some("application/vnd.dataresource+json".to_string()),
                            encoding: None,
                            schema: None,
                        },
                    }),
                },
            );
        }

        if let Some(inputs) = outputs.inputs
            && let Ok(inputs_log) = serde_json::to_value(&inputs)
        {
            result.insert(
                output_keys::INPUTS.to_string(),
                ExecuteResult {
                    output: Output {
                        format: Some(json_format()),
                        transmission_mode: TransmissionMode::Value,
                    },
                    data: InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
                        value: json_input_value(inputs_log),
                        format: Format {
                            media_type: Some("application/json".to_string()),
                            encoding: None,
                            schema: None,
                        },
                    }),
                },
            );
        }

        if let Some(documentation_sources) = outputs.documentation_sources
            && let Ok(value) = documentation_sources.to_input_value()
        {
            result.insert(
                output_keys::DOCUMENTATION_SOURCES.to_string(),
                ExecuteResult {
                    output: Output {
                        format: Some(json_format()),
                        transmission_mode: TransmissionMode::Value,
                    },
                    data: InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
                        value,
                        format: Format {
                            media_type: Some("application/vnd.dataresource+json".to_string()),
                            encoding: None,
                            schema: None,
                        },
                    }),
                },
            );
        }

        if let Some(errors) = outputs.errors {
            result.insert(
                output_keys::ERRORS.to_string(),
                ExecuteResult {
                    output: Output {
                        format: Some(json_format()),
                        transmission_mode: TransmissionMode::Value,
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::Array(errors)),
                },
            );
        }

        result
    }
}

#[derive(Debug, Serialize)]
struct SiteInput {
    location: String,
    /// WKT representation of the input geometry (point or polygon).
    geom: String,
    specification: SiteSpecification,
    /// Whether the original input was derived from a point (`true`) or already a polygon (`false`)
    was_point: bool,
    buffer_size_km: Kilometers,
}

fn feature_id_str(feature: &geojson::Feature) -> String {
    feature
        .id
        .as_ref()
        .map_or("unknown".to_string(), |id| match id {
            geojson::feature::Id::String(s) => s.clone(),
            geojson::feature::Id::Number(n) => n.to_string(),
        })
}

/// Heuristic to extract a location name from a `GeoJSON` feature, checking common properties and the feature ID
fn location_from_feature(
    feature: &geojson::Feature,
    location_property: &str,
) -> anyhow::Result<String> {
    let properties = feature.properties.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Feature `{}` is missing properties",
            feature_id_str(feature)
        )
    })?;

    let Some(value) = properties.get(location_property) else {
        return Err(anyhow::anyhow!(
            "Feature `{}` is missing location property `{}`",
            feature_id_str(feature),
            location_property
        ));
    };

    value.as_str().map(str::to_string).ok_or_else(|| {
        anyhow::anyhow!(
            "Feature `{}` has non-string value for location property `{}`",
            feature_id_str(feature),
            location_property
        )
    })
}

fn site_specification_from_feature(
    feature: &geojson::Feature,
    site_specification_property: &str,
) -> anyhow::Result<SiteSpecification> {
    let properties = feature.properties.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "Feature `{}` is missing properties",
            feature_id_str(feature)
        )
    })?;

    let Some(value) = properties.get(site_specification_property) else {
        return Err(anyhow::anyhow!(
            "Feature `{}` is missing site specification property `{site_specification_property}`",
            feature_id_str(feature),
        ));
    };

    SiteSpecification::from_str(value.as_str().unwrap_or_default()).map_err(|err| {
        anyhow::anyhow!(
            "Feature `{}` has invalid site specification: {err}",
            feature_id_str(feature),
        )
    })
}

/// Convert a point geometry into a minimal polygon by making it empty.
fn polygon_from_point(point: geo_types::Point<f64>) -> geo_types::Polygon<f64> {
    // Create a LineString where all points are the same
    // A closed ring requires at least 4 coordinates usually, so we repeat the same point 4 times to create a valid (but empty) polygon.
    let exterior = geo_types::LineString::from(vec![point.0, point.0, point.0, point.0]);

    geo_types::Polygon::new(exterior, vec![])
}

/// Heuristic to extract a polygon WKT from a `GeoJSON` feature, returning
///  - the polygon geometry,
///  - a minimal polygon at a point location if the input was a point, or
///  - `None` if either no geometry was provided or the geometry is a line string.
fn polygon_from_feature(feature: &geojson::Feature) -> Option<String> {
    use geo_types::{Geometry, MultiPolygon};

    let Some(geometry) = &feature.geometry else {
        return None;
    };

    let geometry = geo_types::Geometry::<f64>::try_from(geometry).ok()?;

    match geometry {
        Geometry::Polygon(polygon) => Some(polygon.wkt_string()),
        Geometry::MultiPolygon(polygons) => Some(polygons.wkt_string()),
        Geometry::Point(point) => Some(polygon_from_point(point).wkt_string()),
        Geometry::MultiPoint(points) => Some(
            points
                .into_iter()
                .map(polygon_from_point)
                .collect::<MultiPolygon<f64>>()
                .wkt_string(),
        ),
        _ => None, // For LineString or other geometries, return None as they are not valid for this process
    }
}

fn was_point(feature: &geojson::Feature) -> bool {
    use geojson::GeometryValue;

    let Some(geometry) = &feature.geometry else {
        return false;
    };

    matches!(
        geometry.value,
        GeometryValue::Point { .. } | GeometryValue::MultiPoint { .. }
    )
}

#[derive(
    Deserialize, Serialize, Debug, PartialEq, AbsDiffEq, JsonSchema, ToSchema, QueryableByName,
)]
#[serde(rename_all = "camelCase")]
#[approx(epsilon_type = f64)]
pub struct SiteRow {
    #[diesel(sql_type = sql_types::Text)]
    #[approx(equal)]
    pub location: String,

    #[diesel(sql_type = sql_types::Nullable<sql_types::Double>)]
    pub area_ha: Option<Hectare>,

    #[diesel(sql_type = sql_types::Double)]
    pub biodiversity_sensitive_area_ha: Hectare,

    #[diesel(sql_type = sql_types::Text)]
    #[approx(equal)]
    pub specification: String,
}

impl From<Vec<SiteRow>> for DataResource<Vec<SiteRow>> {
    fn from(value: Vec<SiteRow>) -> Self {
        Self {
            data: value,
            schema: Fields {
                fields: vec![
                    TableSchemaField {
                        name: "location".into(),
                        r#type: Some(TableSchemaType::String),
                        title: Some("Location".into()),
                    },
                    TableSchemaField {
                        name: "areaHa".into(),
                        r#type: Some(TableSchemaType::Number),
                        title: Some("Area (ha)".into()),
                    },
                    TableSchemaField {
                        name: "biodiversitySensitiveAreaHa".into(),
                        r#type: Some(TableSchemaType::Number),
                        title: Some("Biodiversity Sensitive Area (ha)".into()),
                    },
                    TableSchemaField {
                        name: "specification".into(),
                        r#type: Some(TableSchemaType::String),
                        title: Some("Specification".into()),
                    },
                ],
            },
        }
    }
}

/// Compute the impact metrics from inputs. This helper performs optional HTTP GET requests
/// to any provided `workflow_refs` (if they are HTTP URLs) and includes their responses
/// in the `sources` output for auditing. The spatial computations are placeholders and
/// should be replaced by Geo Engine workflow calls in future.
#[instrument(skip(connection), err(Debug))]
pub async fn compute_biodiversity_sensitive_areas(
    mut connection: PooledConnection<'_>,
    natura2000_schema: &'static str,
    sites: FeatureCollectionGeoJsonInput,
    location_property: &str,
    site_type_property: &str,
) -> anyhow::Result<(Vec<SiteRow>, Vec<String>)> {
    let mut site_inputs: Vec<SiteInput> = Vec::with_capacity(sites.value().features.len());
    let mut errors = Vec::new();

    for feature in &sites.value().features {
        let location = match location_from_feature(feature, location_property) {
            Ok(location) => location,
            Err(error) => {
                errors.push(error.to_string());
                continue;
            }
        };
        let site_specification = match site_specification_from_feature(feature, site_type_property)
        {
            Ok(spec) => spec,
            Err(error) => {
                errors.push(error.to_string());
                continue;
            }
        };
        let Some(polygon) = polygon_from_feature(feature) else {
            errors.push(format!(
                "Skipping feature `{}` due to missing or invalid geometry (not a point or polygon)",
                feature_id_str(feature),
            ));
            continue;
        };

        site_inputs.push(SiteInput {
            location,
            was_point: was_point(feature),
            buffer_size_km: site_specification.buffer_distance(),
            specification: site_specification,
            geom: polygon,
        });
    }

    let site_table: Vec<SiteRow> = sql_query(formatdoc! {r#"
        WITH reference AS (
            SELECT
                v.location,
                v.was_point,
                v.buffer_size_km,
                v.specification,
                ST_Buffer(ST_Transform(('SRID=4326;' || v.geom)::geometry, 3035), v.buffer_size_km * 1_000) AS geom,
                ST_Area(ST_Transform(('SRID=4326;' || v.geom)::geometry, 3035)) AS area_m2
            FROM jsonb_to_recordset($1::jsonb)
                AS v(location text, was_point bool, buffer_size_km double precision, specification text, geom text)
        )
        SELECT
            r.location,
            NULLIF(r.area_m2 / 10_000, 0) AS area_ha,
            -- TODO: or union first over all intersections?
            SUM(ST_Area(ST_Intersection(s.geom, r.geom))) / 10_000 AS biodiversity_sensitive_area_ha,
            'Type: ' || r.specification || E'\n'
                || CASE WHEN r.was_point THEN '(derived from point)' ELSE '' END || E'\n'
                || 'Intersection with ' || r.buffer_size_km || ' km buffer zone to:' || E'\n' 
                || string_agg(s.sitename || ' (' || s.sitecode || ')', ', ') AS specification
        FROM "{natura2000_schema}".naturasite_polygon s, reference r
        WHERE ST_Intersects(s.geom, r.geom)
        GROUP BY r.location, r.was_point, r.buffer_size_km, r.specification, r.area_m2
        ORDER BY biodiversity_sensitive_area_ha DESC
    "#})
    .bind::<sql_types::Json, _>(serde_json::to_value(&site_inputs)?)
    .get_results(&mut *connection)
    .await
    .context(format!("Failed to query {natura2000_schema}.naturasite_polygon"))?;

    Ok((site_table, errors))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CONFIG, db::setup_db, processes::parameters::GeoJsonInputMediaType};
    use approx::assert_abs_diff_eq;
    use diesel_async::SimpleAsyncConnection;
    use geojson::FeatureCollection;
    use indoc::indoc;
    use ogcapi::types::processes::Input;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    async fn mock_db_pool() -> DbPool {
        setup_db(&CONFIG.database).await.unwrap()
    }

    async fn create_schema_and_insert_test_site(
        connection: &mut impl SimpleAsyncConnection,
        schema: &str,
    ) {
        connection
            .batch_execute(&formatdoc! {r#"
            CREATE TABLE "{schema}".naturasite_polygon (
                sitecode TEXT,
                sitename TEXT,
                geom geometry
            );

            INSERT INTO "{schema}".naturasite_polygon (sitecode, sitename, geom)
            VALUES (
                'DE5118301',
                'Dammelsberg und Köhlersgrund',
                ST_GeomFromText('{wkt1}', 3035)
            ), (
                'DE5317307',
                'Fohnbach und Gleibach',
                ST_GeomFromText('{wkt2}', 3035)
            );
            "#,
                wkt1 = include_str!("../../../test-data/DE5118301.wkt"),
                wkt2 = include_str!("../../../test-data/DE5317307.wkt"),
            })
            .await
            .unwrap();
    }

    #[test]
    fn it_deserializes_input() {
        let json = json!({
            "sites": {
                "value": {
                    "type": "FeatureCollection",
                    "features": [
                        {
                            "type": "Feature",
                            "geometry": {
                                "type": "Polygon",
                                "coordinates": [
                                    [
                                        [8.773_665_480_497_84, 50.803_270_291_022_386],
                                        [8.773_649_409_958_182, 50.802_437_463_615_604],
                                        [8.774_613_642_351_255, 50.802_412_072_303_04],
                                        [8.774_597_571_811_597, 50.803_255_056_507_936],
                                        [8.773_665_480_497_84, 50.803_270_291_022_386]
                                    ]
                                ]
                            },
                            "properties": {
                                "location": "Marburger Unistadion",
                                "siteType": "office"
                            }
                        },
                        {
                            "type": "Feature",
                            "geometry": {
                                "type": "Point",
                                "coordinates": [8.770_273_718_309_227, 50.807_468_318_244_67]
                            },
                            "properties": {
                                "location": "Garten des Gedenkens",
                                "siteType": "other"
                            }
                        },
                        {
                            "type": "Feature",
                            "geometry": {
                                "type": "Point",
                                "coordinates": [8.770_273_718_309_227, 50.807_468_318_244_67]
                            },
                            "id": "the-error-feature",
                            "properties": {
                                "siteType": "other"
                            }
                        }
                    ]
                },
                "mediaType": "application/geo+json"
            },
            "locationProperty": "location",
            "siteTypeProperty": "siteType"
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(json).unwrap();

        let json = serde_json::to_value(&inputs).unwrap();

        let _inputs: BiodiversitySensitiveAreasProcessInputs =
            serde_json::from_value(json).unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn process_summary_has_expected_inputs_and_outputs() {
        let pool = mock_db_pool().await;
        create_schema_and_insert_test_site(&mut pool.get().await.unwrap(), &CONFIG.database.schema)
            .await;

        let p = BiodiversitySensitiveAreasProcess::new(pool, &CONFIG.database.schema)
            .await
            .unwrap();
        let process = p.process().expect("to produce process description");

        // summary id / version
        assert_eq!(process.summary.id, "biodiversity-sensitive-areas");
        assert_eq!(process.summary.version, "0.1.0");

        // job control options contain sync and async execute
        let mut has_sync = false;
        let mut has_async = false;
        for opt in &process.summary.job_control_options {
            match opt {
                JobControlOptions::SyncExecute => has_sync = true,
                JobControlOptions::AsyncExecute => has_async = true,
                JobControlOptions::Dismiss => todo!(),
            }
        }
        assert!(has_sync, "expected SyncExecute in job_control_options");
        assert!(has_async, "expected AsyncExecute in job_control_options");

        for key in ["sites", "locationProperty", "siteTypeProperty"] {
            assert!(
                process.inputs.contains_key(key),
                "expected input key `{key}` in process inputs"
            );
        }

        for key in [
            "biodiversitySensitiveAreas",
            "inputs",
            "errors",
            "documentationSources",
        ] {
            assert!(
                process.outputs.contains_key(key),
                "expected output key `{key}` in process outputs"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[allow(
        clippy::too_many_lines,
        reason = "This test is verbose due to the detailed assertions on the process outputs."
    )]
    async fn it_computes_biodiversity_sensitive_areas() {
        let pool = mock_db_pool().await;
        let mut connection = pool.get().await.unwrap();
        create_schema_and_insert_test_site(&mut connection, &CONFIG.database.schema).await;

        let inputs = BiodiversitySensitiveAreasProcessInputs {
            sites: FeatureCollectionGeoJsonInput {
                value: json!({
                  "type": "FeatureCollection",
                  "features": [
                    {
                      "type": "Feature",
                      "geometry": {
                        "type": "Polygon",
                        "coordinates": [
                            [
                                [8.773_665_480_497_84, 50.803_270_291_022_386],
                                [8.773_649_409_958_182, 50.802_437_463_615_604],
                                [8.774_613_642_351_255, 50.802_412_072_303_04],
                                [8.774_597_571_811_597, 50.803_255_056_507_936],
                                [8.773_665_480_497_84, 50.803_270_291_022_386]
                            ]
                        ]
                      },
                      "properties": {
                        "location": "Marbuger Unistadion",
                        "siteType": "office"
                      }
                    },
                    {
                      "type": "Feature",
                      "geometry": {
                        "type": "Point",
                        "coordinates": [8.770_273_718_309_227, 50.807_468_318_244_67]
                      },
                      "properties": {
                        "location": "Garten des Gedenkens",
                        "siteType": "other"
                      }
                    },
                    {
                      "type": "Feature",
                      "geometry": {
                        "type": "Point",
                        "coordinates": [8.770_273_718_309_227, 50.807_468_318_244_67]
                      },
                      "id": "the-error-feature",
                      "properties": {
                        "siteType": "other"
                      }
                    }
                  ]
                })
                .to_string()
                .as_str()
                .parse::<FeatureCollection>()
                .unwrap()
                .into(),
                media_type: GeoJsonInputMediaType::GeoJson,
            },
            location_property: "location".into(),
            site_type_property: "siteType".into(),
        };

        let (site_rows, errors) = compute_biodiversity_sensitive_areas(
            connection,
            CONFIG.database.schema.as_str(),
            inputs.sites,
            inputs.location_property.as_ref(),
            inputs.site_type_property.as_ref(),
        )
        .await
        .unwrap();

        assert_abs_diff_eq!(
            site_rows,
            vec![
                SiteRow {
                    location: "Garten des Gedenkens".into(),
                    area_ha: None,
                    biodiversity_sensitive_area_ha: Hectare(36.307_653_383_984_075),
                    specification: indoc! {"
                        Type: Other
                        (derived from point)
                        Intersection with 20 km buffer zone to:
                        Dammelsberg und Köhlersgrund (DE5118301), Fohnbach und Gleibach (DE5317307)
                    "}
                    .trim_end()
                    .into()
                },
                SiteRow {
                    location: "Marbuger Unistadion".into(),
                    area_ha: Some(Hectare(0.623_035_886_691_041_7)),
                    biodiversity_sensitive_area_ha: Hectare(21.794_987_588_368_837),
                    specification: indoc! {"
                        Type: Office
                        
                        Intersection with 5 km buffer zone to:
                        Dammelsberg und Köhlersgrund (DE5118301)
                    "}
                    .trim_end()
                    .into()
                }
            ],
            epsilon = 1e-6
        );

        assert_eq!(
            errors,
            vec!["Feature `the-error-feature` is missing location property `location`".to_string(),]
        );
    }
}
