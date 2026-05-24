use crate::{
    db::{DbPool, PooledConnection},
    processes::{
        parameters::{
            Area, DataResource, DataResourceSchema, DocumentationSource,
            FeatureCollectionGeoJsonInput, Fields, Kilometers, RelativeJsonPointer, SquareMeter,
            TableSchemaField, TableSchemaItemType, TableSchemaType, UnitForArea,
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

#[doc = include_str!("description.md")]
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
    pub location_name_field: RelativeJsonPointer,

    /// Name of the property in the input `GeoJSON` features that contains the site type information.
    pub site_type_field: RelativeJsonPointer,

    /// Unit for area measurement, with options for hectares (ha) or square meters (m²).
    pub unit_for_area: UnitForArea,
}

mod input_keys {
    pub const SITES: &str = "sites";
    pub const LOCATION_NAME_FIELD: &str = "locationNameField";
    pub const SITE_TYPE_FIELD: &str = "siteTypeField";
    pub const UNIT_FOR_AREA: &str = "unitForArea";
}

#[derive(Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BiodiversitySensitiveAreasProcessOutputs {
    /// Sites in or near biodiversity-sensitive areas
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub biodiversity_sensitive_areas: Option<DataResource<Vec<SiteRowOutput>>>,

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
                description: DescriptionType {
                    title: Some(md_heading(include_str!("description.md")).to_string()),
                    description: md_content(include_str!("description.md"))
                        .to_string()
                        .into(),
                    keywords: vec![
                        "ESG".to_string(),
                        "biodiversity".to_string(),
                        "ESRS E4-5".to_string(),
                        "VSME B5".to_string(),
                    ],
                    ..Default::default()
                },
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
                            title: "Sites".to_string().into(),
                            description: "GeoJSON FeatureCollection of sites to be analyzed."
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
                    input_keys::LOCATION_NAME_FIELD.to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: "Location Name Field".to_string().into(),
                            description: "Reference to the property in the input GeoJSON features that contains the location information."
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
                    input_keys::SITE_TYPE_FIELD.to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: "Site Type Field".to_string().into(),
                            description: "Reference to the property in the input GeoJSON features that contains the site type information."
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
                    input_keys::UNIT_FOR_AREA.to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: "Unit for Area".to_string().into(),
                            description: "Unit for area measurement, with options for hectares (ha) or square meters (m²)."
                                .to_string()
                                .into(),
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<UnitForArea>()
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
                            title: "Biodiversity-sensitive Areas".to_string().into(),
                            description: "Table representation of the identified biodiversity-sensitive areas."
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
                    output_keys::DOCUMENTATION_SOURCES.to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: "Documentation Sources".to_string().into(),
                            description: "List of data sources and workflow references used for audits."
                            .to_string()
                            .into(),
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<DataResource<Vec<DocumentationSource>>>()
                            .to_value(),
                    },
                ),
                (
                    output_keys::ERRORS.to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: "Processing Errors".to_string().into(),
                            description: "List of errors encountered during processing, if any."
                                .to_string()
                                .into(),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<Vec<String>>().to_value(),
                    },
                ),
                (
                    output_keys::INPUTS.to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: "Input Parameters".to_string().into(),
                            description: "Echo of inputs for auditing."
                                .to_string()
                                .into(),
                            ..Default::default()
                        },
                        schema: generator
                            .root_schema_for::<BiodiversitySensitiveAreasProcessInputs>()
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
                inputs.location_name_field.as_ref(),
                inputs.site_type_field.as_ref(),
                inputs.unit_for_area,
            )
            .await?;

            outputs.biodiversity_sensitive_areas = execute
                .outputs
                .contains_key(output_keys::BIODIVERSITY_SENSITIVE_AREAS)
                .then_some(site_row_into_output(site_table, inputs.unit_for_area));

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
    buffer_distance_km: Kilometers,
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

#[derive(Deserialize, Serialize, Debug, PartialEq, QueryableByName)]
#[serde(rename_all = "camelCase")]
pub struct SiteRow {
    #[diesel(sql_type = sql_types::Text)]
    pub location: String,

    #[diesel(sql_type = sql_types::Nullable<sql_types::Double>)]
    pub area_m2: Option<SquareMeter>,

    #[diesel(sql_type = sql_types::Bool)]
    pub site_in_biodiversity_sensitive_area: bool,

    #[diesel(sql_type = sql_types::Bool)]
    pub site_near_biodiversity_sensitive_area: bool,

    #[diesel(sql_type = sql_types::Double)]
    pub biodiversity_sensitive_area_m2: SquareMeter,

    #[diesel(sql_type = sql_types::Nullable<sql_types::Array<sql_types::Text>>)]
    pub intersecting_biodiversity_sensitive_areas: Option<Vec<String>>,

    #[diesel(sql_type = sql_types::Array<sql_types::Text>)]
    pub nearby_biodiversity_sensitive_areas: Vec<String>,

    #[diesel(sql_type = sql_types::Text)]
    pub site_type: String,

    #[diesel(sql_type = sql_types::Double)]
    pub buffer_distance_km: Kilometers,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, AbsDiffEq, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
#[approx(epsilon_type = f64)]
pub struct SiteRowOutput {
    #[approx(equal)]
    pub location: String,

    pub area: Option<Area>,

    #[approx(equal)]
    pub site_in_biodiversity_sensitive_area: bool,

    #[approx(equal)]
    pub site_near_biodiversity_sensitive_area: bool,

    pub biodiversity_sensitive_area: Area,

    #[approx(equal)]
    pub specification: String,

    #[approx(equal)]
    pub intersecting_biodiversity_sensitive_areas: Vec<String>,

    #[approx(equal)]
    pub nearby_biodiversity_sensitive_areas: Vec<String>,
}

fn site_row_into_output(
    site_rows: Vec<SiteRow>,
    unit_for_area: UnitForArea,
) -> DataResource<Vec<SiteRowOutput>> {
    DataResource {
        name: "Biodiversity-sensitive Areas".to_string(),
        data: site_rows
            .into_iter()
            .map(|row| SiteRowOutput {
                location: row.location,
                area: row
                    .area_m2
                    .map(|a| Area::from_square_meters(a, unit_for_area)),
                site_in_biodiversity_sensitive_area: row.site_in_biodiversity_sensitive_area,
                site_near_biodiversity_sensitive_area: row.site_near_biodiversity_sensitive_area,
                biodiversity_sensitive_area: Area::from_square_meters(
                    row.biodiversity_sensitive_area_m2,
                    unit_for_area,
                ),
                specification: format!(
                    "Type \"{site_type}\"{was_point} with buffer distance of {buffer_distance_km}",
                    site_type = row.site_type,
                    buffer_distance_km = row.buffer_distance_km,
                    was_point = row.area_m2.map(|_| " (was point)").unwrap_or_default(),
                ),
                intersecting_biodiversity_sensitive_areas: row
                    .intersecting_biodiversity_sensitive_areas
                    .unwrap_or_default(),
                nearby_biodiversity_sensitive_areas: row.nearby_biodiversity_sensitive_areas,
            })
            .collect(),
        schema: Fields {
            fields: vec![
                TableSchemaField {
                    name: "location".into(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Location".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "area".into(),
                    r#type: Some(TableSchemaType::Number),
                    title: Some(format!("Area ({unit_for_area})")),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "siteInBiodiversitySensitiveArea".into(),
                    r#type: Some(TableSchemaType::Boolean),
                    title: Some("Site in Biodiversity-sensitive Area".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "siteNearBiodiversitySensitiveArea".into(),
                    r#type: Some(TableSchemaType::Boolean),
                    title: Some("Site near Biodiversity-sensitive Area".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "biodiversitySensitiveArea".into(),
                    r#type: Some(TableSchemaType::Number),
                    title: Some(format!("Biodiversity Sensitive Area ({unit_for_area})")),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "specification".into(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Specification".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "intersectingBiodiversitySensitiveAreas".into(),
                    r#type: Some(TableSchemaType::List),
                    title: Some("Intersecting Biodiversity-sensitive Areas".into()),
                    item_type: Some(TableSchemaItemType::String),
                },
                TableSchemaField {
                    name: "nearbyBiodiversitySensitiveAreas".into(),
                    r#type: Some(TableSchemaType::List),
                    title: Some("Nearby Biodiversity-sensitive Areas".into()),
                    item_type: Some(TableSchemaItemType::String),
                },
            ],
        },
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
    unit_for_area: UnitForArea,
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
            buffer_distance_km: site_specification.buffer_distance(),
            specification: site_specification,
            geom: polygon,
        });
    }

    let site_table: Vec<SiteRow> = sql_query(formatdoc! {r#"
        WITH reference AS (
            SELECT
                v.location,
                v.was_point,
                v.buffer_distance_km,
                v.specification,
                ST_Transform(('SRID=4326;' || v.geom)::geometry, 3035) AS geom,
                ST_Buffer(ST_Transform(('SRID=4326;' || v.geom)::geometry, 3035), v.buffer_distance_km * 1_000) AS buffered_geom,
                ST_Area(ST_Transform(('SRID=4326;' || v.geom)::geometry, 3035)) AS area_m2
            FROM jsonb_to_recordset($1::jsonb)
                AS v(location text, was_point bool, buffer_distance_km double precision, specification text, geom text)
        )
        SELECT
            r.location,
            r.buffer_distance_km as buffer_distance_km,
            r.specification AS site_type,
            NULLIF(r.area_m2, 0) AS area_m2,
            COALESCE(ST_AREA(ST_INTERSECTION(s_in.geom, r.geom)), 0) > 0 AS site_in_biodiversity_sensitive_area,
            ST_AREA(ST_INTERSECTION(s_out.geom, r.buffered_geom)) > 0 AS site_near_biodiversity_sensitive_area,
            ST_AREA(ST_INTERSECTION(s_out.geom, r.buffered_geom)) AS biodiversity_sensitive_area_m2,
            s_in.site_list AS intersecting_biodiversity_sensitive_areas,
            s_out.site_list AS nearby_biodiversity_sensitive_areas
        FROM reference r
        LEFT JOIN (
            SELECT
                r.location,
                ST_COLLECT(s.geom) as geom,
                ARRAY_AGG(s.sitename || ' (' || s.sitecode || ')') as site_list
            FROM "{natura2000_schema}".naturasite_polygon s, reference r
            WHERE ST_Intersects(s.geom, r.geom)
            GROUP BY r.location
        ) as s_in USING (location)
        JOIN (
            SELECT
                r.location,
                ST_COLLECT(s.geom) as geom,
                ARRAY_AGG(s.sitename || ' (' || s.sitecode || ')') as site_list
            FROM "{natura2000_schema}".naturasite_polygon s, reference r
            WHERE ST_Intersects(s.geom, r.buffered_geom)
            GROUP BY r.location
        ) as s_out USING (location)
        ORDER BY biodiversity_sensitive_area_m2 DESC;
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
    use crate::{
        CONFIG,
        db::setup_db,
        processes::parameters::{GeoJsonInputMediaType, Hectare},
    };
    use approx::abs_diff_ne;
    use diesel_async::SimpleAsyncConnection;
    use geojson::FeatureCollection;
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
            "locationNameField": "location",
            "siteTypeField": "siteType",
            "unitForArea": "m²"
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

        for key in [
            input_keys::SITES,
            input_keys::LOCATION_NAME_FIELD,
            input_keys::SITE_TYPE_FIELD,
            input_keys::UNIT_FOR_AREA,
        ] {
            assert!(
                process.inputs.contains_key(key),
                "expected input key `{key}` in process inputs"
            );
        }

        for key in [
            output_keys::BIODIVERSITY_SENSITIVE_AREAS,
            output_keys::INPUTS,
            output_keys::ERRORS,
            output_keys::DOCUMENTATION_SOURCES,
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

        // crate::util::setup_tracing(
        //     crate::config::Logging {
        //         level: crate::config::LogLevel::Debug,
        //     }
        //     .into(),
        // );

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
                        "type": "Polygon",
                        "coordinates": [
                          [
                            [8.754_139_485_384, 50.809_101_655_468],
                            [8.754_266_459_025, 50.808_497_270_648],
                            [8.755_374_371_521, 50.809_035_957_506],
                            [8.754_139_485_384, 50.809_101_655_468]
                          ]
                        ]
                      },
                      "properties": {
                        "location": "Auf dem Dammelsberg",
                        "siteType": "office"
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
            location_name_field: "location".into(),
            site_type_field: "siteType".into(),
            unit_for_area: UnitForArea::Hectare,
        };

        let (site_rows, errors) = compute_biodiversity_sensitive_areas(
            connection,
            CONFIG.database.schema.as_str(),
            inputs.sites,
            inputs.location_name_field.as_ref(),
            inputs.site_type_field.as_ref(),
            inputs.unit_for_area,
        )
        .await
        .unwrap();

        let site_row_outputs = site_row_into_output(site_rows, inputs.unit_for_area);

        let expected = vec![
            SiteRowOutput {
                location: "Garten des Gedenkens".into(),
                area: None,
                site_in_biodiversity_sensitive_area: false,
                site_near_biodiversity_sensitive_area: true,
                biodiversity_sensitive_area: Area::Hectare(Hectare(36.307_653_383_984_075)),
                specification: "Type \"Other\" with buffer distance of 20 km".into(),
                intersecting_biodiversity_sensitive_areas: vec![],
                nearby_biodiversity_sensitive_areas: vec![
                    "Dammelsberg und Köhlersgrund (DE5118301)".into(),
                    "Fohnbach und Gleibach (DE5317307)".into(),
                ],
            },
            SiteRowOutput {
                location: "Auf dem Dammelsberg".into(),
                area: Some(Area::Hectare(Hectare(0.289_339_552_615_731_47))),
                site_in_biodiversity_sensitive_area: true,
                site_near_biodiversity_sensitive_area: true,
                biodiversity_sensitive_area: Area::Hectare(Hectare(21.794_987_588_368_837)),
                specification: "Type \"Office\" (was point) with buffer distance of 5 km".into(),
                intersecting_biodiversity_sensitive_areas: vec![
                    "Dammelsberg und Köhlersgrund (DE5118301)".into(),
                ],
                nearby_biodiversity_sensitive_areas: vec![
                    "Dammelsberg und Köhlersgrund (DE5118301)".into(),
                ],
            },
            SiteRowOutput {
                location: "Marburger Unistadion".into(),
                area: Some(Area::Hectare(Hectare(0.623_035_886_691_041_7))),
                site_in_biodiversity_sensitive_area: false,
                site_near_biodiversity_sensitive_area: true,
                biodiversity_sensitive_area: Area::Hectare(Hectare(21.794_987_588_368_837)),
                specification: "Type \"Office\" (was point) with buffer distance of 5 km".into(),
                intersecting_biodiversity_sensitive_areas: vec![],
                nearby_biodiversity_sensitive_areas: vec![
                    "Dammelsberg und Köhlersgrund (DE5118301)".into(),
                ],
            },
        ];

        if abs_diff_ne!(site_row_outputs.data, expected, epsilon = 1e-6,) {
            assert_eq!(site_row_outputs.data, expected); // pretty assertions
        }

        assert_eq!(
            errors,
            vec!["Feature `the-error-feature` is missing location property `location`".to_string(),]
        );
    }
}
