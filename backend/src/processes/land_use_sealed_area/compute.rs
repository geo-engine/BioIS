use crate::{
    CONFIG,
    processes::{
        land_use_sealed_area::types::{
            LandUseSummary, LandUseSummaryRow, LandUseSummaryRowType, SiteLandUseRow,
            SiteSpecification, TypedPreviousLandUseSummary,
        },
        parameters::{
            Area, BoundingBox, DocumentationSource, FeatureCollectionGeoJsonInput,
            GeoJsonFeatureCollection, SquareMeter, Year, geojson_feature_collection_utils,
            geojson_feature_utils,
        },
        util::{
            raster_result_descriptor, vector_result_descriptor, year_range_from_time_descriptor,
        },
    },
    util::{spawn_blocking, to_api_raster_process, to_api_vector_process},
};
use anyhow::{Context, Result};
use geoengine_api_client::{
    apis::{
        configuration::Configuration,
        datasets_api::create_dataset_handler,
        ogcwfs_api::wfs_handler,
        uploads_api::upload_handler,
        workflows_api::{
            get_workflow_metadata_handler, get_workflow_provenance_handler,
            register_workflow_handler,
        },
    },
    models::{
        AddDataset, ClassificationMeasurement, ColumnNames, ContinuousMeasurement, CreateDataset,
        DataPath, DataPathUpload, DatasetDefinition, DeriveOutRasterSpecsSource, Expression,
        ExpressionParameters, FeatureAggregationMethod, FeatureDataType, GdalSource,
        GdalSourceParameters, GeoJson, Measurement, MetaDataDefinition, Names, NewOutputColumn,
        OgrMetaData, OgrSource, OgrSourceColumnSpec, OgrSourceDataset, OgrSourceErrorSpec,
        OgrSourceParameters, OutputColumn, RasterBandDescriptor, RasterDataType, RasterOperator,
        RasterVectorJoin, RasterVectorJoinParameters, Reprojection, ReprojectionParameters,
        SingleRasterOrVectorOperator, SingleRasterOrVectorSource, SingleRasterSource,
        SingleVectorMultipleRasterSources, SingleVectorSource, TemporalAggregationMethod,
        VectorColumnInfo, VectorDataType, VectorExpression, VectorExpressionParameters,
        VectorOperator, VectorResultDescriptor, WfsRequest, WfsService,
    },
};
use geojson::{Feature, GeometryValue, Position};
use ogcapi::types::common::Crs;
use std::{path::PathBuf, str::FromStr, sync::LazyLock};
use tracing::instrument;

pub static IMPERVIOUSNESS_CRS: LazyLock<Crs> = LazyLock::new(|| Crs::from_epsg(3035));
pub const FRACTION_SEALED_COLUMN_NAME: &str = "fractionSealed";
pub const AREA_COLUMN_NAME: &str = "area";

/// Compute land-use data for each site including sealed areas and nature-oriented areas.
///
/// This function processes the input sites and calculates per-site land-use metrics.
///
/// Returns a tuple of (per-site rows, error messages).
#[instrument(skip(configuration, sites), err(Debug))]
pub async fn compute_site_land_use_data(
    configuration: &Configuration,
    year: Year,
    sites: &FeatureCollectionGeoJsonInput,
    location_name_field: &str,
    location_type_field: &str,
) -> anyhow::Result<(Vec<SiteLandUseRow>, Vec<String>)> {
    let mut site_rows = Vec::new();

    tracing::info!(
        features_count = sites.value().features.len(),
        "Computing land-use data for individual sites"
    );

    let mut validation_and_bbox =
        validate_and_extract_bbox(sites, location_name_field, location_type_field)?;

    let upload_data_id = upload_geo_json(
        configuration,
        sites,
        location_name_field,
        location_type_field,
        &validation_and_bbox.bbox,
    )
    .await?;

    let result_geojson = GeoJsonFeatureCollection::try_from(
        sealed_area_process(configuration, upload_data_id, year).await?,
    )?;

    for feature in &result_geojson.as_ref().features {
        match extract_site_land_use_rows(feature, location_name_field, location_type_field) {
            Ok(row) => site_rows.push(row),
            Err(err) => validation_and_bbox.errors.push(format!(
                "Feature `{id}`: {err}",
                id = geojson_feature_utils::id_str(feature)
            )),
        }
    }

    Ok((site_rows, validation_and_bbox.errors))
}

fn extract_site_land_use_rows(
    feature: &Feature,
    location_name_field: &str,
    location_type_field: &str,
) -> Result<SiteLandUseRow> {
    let area = SquareMeter(geojson_feature_utils::get_number(
        feature,
        AREA_COLUMN_NAME,
    )?);
    Ok(SiteLandUseRow {
        location: geojson_feature_utils::get_string(feature, location_name_field)?,
        land_use_type: geojson_feature_get_site_type(feature, location_type_field)?,
        sealed_area: area
            * geojson_feature_utils::get_number(feature, FRACTION_SEALED_COLUMN_NAME)?,
        area,
    })
}

#[instrument(skip_all, err(Debug))]
async fn upload_geo_json(
    configuration: &Configuration,
    geo_json: &FeatureCollectionGeoJsonInput,
    location_name_field: &str,
    location_type_field: &str,
    bbox: &BoundingBox,
) -> Result<String> {
    let geo_json_str = serde_json::to_string(&geo_json.value())?;

    let (temp_file_stem, temp_file_suffix) = ("geo", ".json");
    let temp_file_name = format!("{temp_file_stem}{temp_file_suffix}");
    let (temp_dir, temp_path, temp_file_name) =
        spawn_blocking(move || -> Result<(tempfile::TempDir, PathBuf, String)> {
            let temp_dir = tempfile::tempdir()?;

            let temp_path = temp_dir.path().join(&temp_file_name);
            std::io::Write::write_all(
                &mut std::fs::File::create(&temp_path)?,
                geo_json_str.as_bytes(),
            )?;

            Ok((temp_dir, temp_path, temp_file_name))
        })
        .await??;

    let upload_id = upload_handler(configuration, vec![temp_path]).await?.id;

    drop(temp_dir); // temp_dir is deleted after upload

    let create_dataset_request = CreateDataset {
        data_path: Box::new(DataPath::DataPathUpload(Box::new(DataPathUpload {
            upload: upload_id,
        }))),
        definition: Box::new(DatasetDefinition {
            properties: Box::new(AddDataset {
                name: None,
                display_name: "Uploaded GeoJSON".to_string(),
                description: String::new(),
                source_operator: "OgrSource".to_string(),
                symbology: None,
                provenance: None,
                tags: None,
            }),
            meta_data: Box::new(MetaDataDefinition::OgrMetaData(Box::new(OgrMetaData {
                r#type: Default::default(),
                loading_info: Box::new(OgrSourceDataset {
                    file_name: temp_file_name.clone(),
                    layer_name: geojson_feature_collection_utils::name(geo_json.value())
                        .unwrap_or(temp_file_stem)
                        .to_string(),
                    data_type: Some(Some(VectorDataType::MultiPolygon)),
                    time: None,
                    default_geometry: None,
                    columns: Some(Some(Box::new(OgrSourceColumnSpec {
                        format_specifics: None,
                        x: String::new(),
                        y: None,
                        int: None,
                        float: None,
                        text: Some(vec![
                            location_name_field.to_string(),
                            location_type_field.to_string(),
                        ]),
                        bool: None,
                        datetime: None,
                        rename: None,
                    }))),
                    force_ogr_time_filter: None,
                    force_ogr_spatial_filter: None,
                    on_error: OgrSourceErrorSpec::Ignore,
                    sql_query: None,
                    attribute_query: None,
                    cache_ttl: None,
                }),
                result_descriptor: Box::new(VectorResultDescriptor {
                    data_type: VectorDataType::MultiPolygon,
                    spatial_reference: bbox.crs().as_known_crs(),
                    columns: [
                        (
                            location_name_field.to_string(),
                            VectorColumnInfo {
                                data_type: FeatureDataType::Text,
                                measurement: Box::new(Measurement::Unitless(Default::default())),
                            },
                        ),
                        (
                            location_type_field.to_string(),
                            VectorColumnInfo {
                                data_type: FeatureDataType::Text,
                                measurement: Box::new(Measurement::Unitless(Default::default())),
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    time: None,
                    bbox: Some(Some(Box::new(bbox.to_bounding_box_2d()))),
                }),
            }))),
        }),
    };

    let dataset_response = create_dataset_handler(configuration, create_dataset_request).await?;

    Ok(dataset_response.dataset_name)
}

#[derive(Debug)]
struct BboxAndErrors {
    bbox: BoundingBox,
    errors: Vec<String>,
}

impl BboxAndErrors {
    fn update_bbox<'p>(&mut self, positions: impl Iterator<Item = &'p Position>) {
        self.bbox.enlarge_by_positions(positions);
    }
}

#[instrument(skip_all, err(Debug))]
#[allow(
    clippy::needless_continue,
    reason = "keeping the last continue statement for clarity and future-proofing"
)]
fn validate_and_extract_bbox(
    geo_json: &FeatureCollectionGeoJsonInput,
    location_name_field: &str,
    location_type_field: &str,
) -> Result<BboxAndErrors> {
    let mut result = BboxAndErrors {
        bbox: BoundingBox::new_invalid(Crs::from_epsg(4326)),
        errors: Vec::new(),
    };

    for feature in &geo_json.value().features {
        let Some(geometry) = &feature.geometry else {
            result.errors.push(format!(
                "Feature `{id}` has no geometry",
                id = geojson_feature_utils::id_str(feature)
            ));
            continue;
        };

        match &geometry.value {
            GeometryValue::Polygon { coordinates } => {
                result.update_bbox(coordinates.iter().flatten());
            }
            GeometryValue::MultiPolygon { coordinates } => {
                result.update_bbox(coordinates.iter().flatten().flatten());
            }
            _ => {
                result.errors.push(format!(
                    "Feature `{id}` has unsupported geometry type: {geom_type}",
                    id = geojson_feature_utils::id_str(feature),
                    geom_type = geometry.value.type_name()
                ));
                continue;
            }
        }

        if let Err(err) =
            geojson_feature_utils::check_property_is_string(feature, location_name_field)
        {
            result.errors.push(err.to_string());
            continue;
        }

        if let Err(err) = check_property_is_site_type(feature, location_type_field) {
            result.errors.push(err.to_string());
            continue;
        }
    }

    if result.errors.len() == geo_json.value().features.len() {
        return Err(anyhow::anyhow!(
            "All features have errors, cannot compute bounding box"
        ));
    }

    Ok(result)
}

fn check_property_is_site_type(feature: &Feature, field: &str) -> Result<()> {
    let value_str = geojson_feature_utils::get_str(feature, field)?;

    SiteSpecification::from_str(value_str)
        .context(format!(
            "Feature `{id}` has invalid type (expected {expected})",
            id = geojson_feature_utils::id_str(feature),
            expected = SiteSpecification::EXPECTED
        ))
        .map(|_| ())
}

fn geojson_feature_get_site_type(feature: &Feature, field: &str) -> Result<SiteSpecification> {
    let value_str = geojson_feature_utils::get_string(feature, field)?;
    SiteSpecification::from_str(&value_str).context(format!(
        "Feature `{id}` has invalid type",
        id = geojson_feature_utils::id_str(feature)
    ))
}

#[instrument(skip(configuration), err(Debug))]
async fn sealed_area_process(
    configuration: &Configuration,
    upload_data_id: String,
    year: Year,
) -> Result<GeoJson> {
    let operators = build_sealed_area_vector_operator(upload_data_id);

    let processing_graph_id = register_workflow_handler(configuration, operators.sealed_area())
        .await?
        .id
        .to_string();

    let time_str = format!("{year}-01-01T00:00:00Z");

    let locations_projected_id =
        register_workflow_handler(configuration, operators.locations_projected())
            .await?
            .id
            .to_string();

    let bbox = BoundingBox::try_from(&vector_result_descriptor(
        get_workflow_metadata_handler(configuration, &locations_projected_id)
            .await
            .context("unable to get workflow metadata")?,
    )?)?;

    wfs_handler(
        configuration,
        &processing_graph_id,
        WfsRequest::GetFeature,
        Some(&bbox.wfs_string()),
        None,
        None,
        None,
        None,
        None,
        Some(WfsService::Wfs),
        None,
        Some(&IMPERVIOUSNESS_CRS.as_known_crs()),
        Some(&time_str),
        Some(&processing_graph_id),
        None,
    )
    .await
    .context("unable to compute sealed areas")
}

/// Compute summary table from per-site land-use data.
///
/// This function aggregates the per-site rows into a summary table with totals
/// for each land-use type. If per-site data includes previous year comparisons,
/// the summary will also include aggregated year-over-year changes.
#[instrument(skip_all)]
pub fn compute_summary_from_sites(
    site_rows: &[SiteLandUseRow],
    previous_year: TypedPreviousLandUseSummary,
) -> LandUseSummary {
    let mut summary = LandUseSummary {
        total_sealed_area: LandUseSummaryRow {
            land_use_type: LandUseSummaryRowType::TotalSealedArea,
            previous_year: previous_year.total_sealed_area.map(Area::to_square_meters),
            reporting_year: SquareMeter(0.0),
            percentage_change: None,
        },
        total_nature_on_site_area: LandUseSummaryRow {
            land_use_type: LandUseSummaryRowType::TotalNatureOnSiteArea,
            previous_year: previous_year
                .total_nature_on_site_area
                .map(Area::to_square_meters),
            reporting_year: SquareMeter(0.0),
            percentage_change: None,
        },
        total_nature_off_site_area: LandUseSummaryRow {
            land_use_type: LandUseSummaryRowType::TotalNatureOffSiteArea,
            previous_year: previous_year
                .total_nature_off_site_area
                .map(Area::to_square_meters),
            reporting_year: SquareMeter(0.0),
            percentage_change: None,
        },
        total_use_of_land: LandUseSummaryRow {
            land_use_type: LandUseSummaryRowType::TotalUseOfLand,
            previous_year: previous_year.total_use_of_land.map(Area::to_square_meters),
            reporting_year: SquareMeter(0.0),
            percentage_change: None,
        },
    };

    for row in site_rows {
        match row.land_use_type {
            SiteSpecification::Site => summary.total_sealed_area.reporting_year += row.sealed_area,
            SiteSpecification::NatureOnSite => {
                summary.total_nature_on_site_area.reporting_year += row.area;
            }
            SiteSpecification::NatureOffSite => {
                summary.total_nature_off_site_area.reporting_year += row.area;
            }
        }
        summary.total_use_of_land.reporting_year += row.area;
    }

    // Calculate percentage changes if previous year data is available
    summary.total_sealed_area.percentage_change = calculate_percentage_change(
        summary.total_sealed_area.previous_year,
        summary.total_sealed_area.reporting_year,
    );
    summary.total_nature_on_site_area.percentage_change = calculate_percentage_change(
        summary.total_nature_on_site_area.previous_year,
        summary.total_nature_on_site_area.reporting_year,
    );
    summary.total_nature_off_site_area.percentage_change = calculate_percentage_change(
        summary.total_nature_off_site_area.previous_year,
        summary.total_nature_off_site_area.reporting_year,
    );
    summary.total_use_of_land.percentage_change = calculate_percentage_change(
        summary.total_use_of_land.previous_year,
        summary.total_use_of_land.reporting_year,
    );

    summary
}

/// Helper function to calculate percentage change between two areas.
/// If the previous value is None or zero, returns None to avoid division by zero.
fn calculate_percentage_change(previous: Option<SquareMeter>, current: SquareMeter) -> Option<f64> {
    match previous {
        None | Some(SquareMeter(0.0)) => None, // Avoid division by zero
        Some(previous) => Some(((current - previous) / previous) * 100.),
    }
}

struct ComputeOperators {
    sealed_area: VectorOperator,
    locations_projected: VectorOperator,
}

impl ComputeOperators {
    fn sealed_area(&self) -> geoengine_api_client::models::Workflow {
        to_api_vector_process(&self.sealed_area)
    }

    fn locations_projected(&self) -> geoengine_api_client::models::Workflow {
        to_api_vector_process(&self.locations_projected)
    }
}

/// Builds a vector operator pipeline for projecting the uploaded locations to the imperviousness CRS.
fn projected_locations_operator(upload_data_id: String) -> VectorOperator {
    let locations = VectorOperator::OgrSource(Box::new(OgrSource {
        r#type: Default::default(),
        params: OgrSourceParameters {
            data: upload_data_id,
            attribute_projection: None,
        }
        .into(),
    }));

    VectorOperator::Reprojection(Box::new(Reprojection {
        r#type: Default::default(),
        params: Box::new(ReprojectionParameters {
            derive_out_spec: Some(DeriveOutRasterSpecsSource::DataBounds),
            target_spatial_reference: IMPERVIOUSNESS_CRS.as_known_crs(),
        }),
        sources: Box::new(SingleRasterOrVectorSource {
            source: Box::new(SingleRasterOrVectorOperator::VectorOperator(Box::new(
                locations,
            ))),
        }),
    }))
}

fn imperviousness_raster_operator() -> RasterOperator {
    RasterOperator::GdalSource(Box::new(GdalSource {
        r#type: Default::default(),
        params: GdalSourceParameters {
            data: CONFIG.data_ids.land_use_imperviousness_builtup.clone(),
            overview_level: None,
        }
        .into(),
    }))
}

/// Builds a vector operator pipeline for sealed area analysis.
///
/// The pipeline combines vector data from the given upload with imperviousness raster data.
/// It applies the following stages:
/// 1. Load imperviousness raster via `GdalSource`
/// 2. Join with vector data via `RasterVectorJoin`
#[instrument(skip_all)]
fn build_sealed_area_vector_operator(upload_data_id: String) -> ComputeOperators {
    let locations_projected = projected_locations_operator(upload_data_id);

    let imperviousness = imperviousness_raster_operator();

    let imperviousness_classification = RasterOperator::Expression(Box::new(Expression {
        r#type: Default::default(),
        params: ExpressionParameters {
            expression: "if A <= 1 { A } else { NODATA }".into(),
            map_no_data: false,
            output_band: Some(Box::new(RasterBandDescriptor {
                name: "sealed".into(),
                measurement: Box::new(Measurement::Classification(Box::new(
                    ClassificationMeasurement {
                        r#type: Default::default(),
                        classes: [
                            ("0".into(), "not_sealed".into()),
                            ("1".into(), "sealed".into()),
                        ]
                        .into_iter()
                        .collect(),
                        measurement: "classification".into(),
                    },
                ))),
            })),
            output_type: RasterDataType::U8,
        }
        .into(),
        sources: Box::new(SingleRasterSource {
            raster: Box::new(imperviousness),
        }),
    }));

    // Combine vector and raster data
    let locations_with_sealed_area = VectorOperator::RasterVectorJoin(Box::new(RasterVectorJoin {
        r#type: Default::default(),
        params: RasterVectorJoinParameters {
            names: ColumnNames::Names(Box::new(Names {
                r#type: Default::default(),
                values: vec![FRACTION_SEALED_COLUMN_NAME.to_string()],
            }))
            .into(),
            feature_aggregation: FeatureAggregationMethod::Mean,
            feature_aggregation_ignore_no_data: Some(true),
            temporal_aggregation: TemporalAggregationMethod::None,
            temporal_aggregation_ignore_no_data: Some(true),
        }
        .into(),
        sources: SingleVectorMultipleRasterSources {
            vector: Box::new(locations_projected.clone()),
            rasters: vec![imperviousness_classification],
        }
        .into(),
    }));

    let sealed_area = VectorOperator::VectorExpression(Box::new(VectorExpression {
        r#type: Default::default(),
        params: Box::new(VectorExpressionParameters {
            input_columns: vec![],
            expression: "area(geom)".into(),
            output_column: Box::new(OutputColumn::NewOutputColumn(Box::new(NewOutputColumn {
                r#type: Default::default(),
                value: AREA_COLUMN_NAME.to_string(),
            }))),
            geometry_column_name: Some("geom".into()),
            output_measurement: Some(Box::new(Measurement::Continuous(Box::new(
                ContinuousMeasurement {
                    r#type: Default::default(),
                    measurement: "area".into(),
                    unit: Some(Some("m²".into())),
                },
            )))),
        }),
        sources: Box::new(SingleVectorSource {
            vector: Box::new(locations_with_sealed_area),
        }),
    }));

    ComputeOperators {
        sealed_area,
        locations_projected,
    }
}

/// Computes the available time range from the imperviousness raster.
#[instrument(skip_all, err(Debug))]
pub async fn compute_available_time_range_from_imperviousness_raster(
    configuration: &Configuration,
) -> Result<(Year, Year)> {
    let operator = to_api_raster_process(&imperviousness_raster_operator());
    let processing_graph_id = register_workflow_handler(configuration, operator)
        .await?
        .id
        .to_string();
    let result_descriptor = raster_result_descriptor(
        get_workflow_metadata_handler(configuration, &processing_graph_id)
            .await
            .context("unable to get process metadata")?,
    )?;

    year_range_from_time_descriptor(&result_descriptor.time)
}

/// Computes the [`DocumentationSource`]s for the imperviousness raster.
#[instrument(skip_all, err(Debug))]
pub async fn compute_documentation_sources(
    configuration: &Configuration,
) -> Result<Vec<DocumentationSource>> {
    let operator = to_api_raster_process(&imperviousness_raster_operator());
    let processing_graph_id = register_workflow_handler(configuration, operator)
        .await?
        .id
        .to_string();
    let provenance = get_workflow_provenance_handler(configuration, &processing_graph_id)
        .await
        .context("unable to get process provenance")?;

    Ok(provenance
        .into_iter()
        .map(DocumentationSource::from)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processes::parameters::UnitForArea;

    #[test]
    fn it_calculates_percentage_change() {
        // Normal case: positive change
        let result = calculate_percentage_change(Some(SquareMeter(100.0)), SquareMeter(150.0));
        assert_eq!(result, Some(50.0));

        // Normal case: negative change
        let result = calculate_percentage_change(Some(SquareMeter(150.0)), SquareMeter(100.0));
        assert_eq!(result, Some(-33.333_333_333_333_33));

        // No previous data
        let result = calculate_percentage_change(None, SquareMeter(100.0));
        assert_eq!(result, None);

        // Previous data is zero
        let result = calculate_percentage_change(Some(SquareMeter(0.0)), SquareMeter(100.0));
        assert_eq!(result, None);
    }

    #[test]
    fn it_computes_summary_from_sites() {
        let site_rows = vec![
            SiteLandUseRow {
                location: "Site A".to_string(),
                land_use_type: SiteSpecification::Site,
                area: SquareMeter(1000.0),
                sealed_area: SquareMeter(500.0),
            },
            SiteLandUseRow {
                location: "Site B".to_string(),
                land_use_type: SiteSpecification::Site,
                area: SquareMeter(2000.0),
                sealed_area: SquareMeter(800.0),
            },
            SiteLandUseRow {
                location: "Nature1".to_string(),
                land_use_type: SiteSpecification::NatureOnSite,
                area: SquareMeter(300.0),
                sealed_area: SquareMeter(0.0),
            },
            SiteLandUseRow {
                location: "Nature2".to_string(),
                land_use_type: SiteSpecification::NatureOffSite,
                area: SquareMeter(200.0),
                sealed_area: SquareMeter(0.0),
            },
        ];

        let previous = TypedPreviousLandUseSummary {
            total_sealed_area: Some(Area::new(1000.0, UnitForArea::SquareMeter)),
            total_nature_on_site_area: Some(Area::new(250.0, UnitForArea::SquareMeter)),
            total_nature_off_site_area: None,
            total_use_of_land: Some(Area::new(3000.0, UnitForArea::SquareMeter)),
        };

        let summary = compute_summary_from_sites(&site_rows, previous);

        // Sealed area: 500 + 800 = 1300, previous = 1000
        assert_eq!(
            summary.total_sealed_area.reporting_year,
            SquareMeter(1300.0)
        );
        assert_eq!(summary.total_sealed_area.percentage_change, Some(30.0));

        // Nature on-site: 300, previous = 250
        assert_eq!(
            summary.total_nature_on_site_area.reporting_year,
            SquareMeter(300.0)
        );
        assert_eq!(
            summary.total_nature_on_site_area.percentage_change,
            Some(20.0)
        );

        // Nature off-site: 200, no previous data
        assert_eq!(
            summary.total_nature_off_site_area.reporting_year,
            SquareMeter(200.0)
        );
        assert_eq!(summary.total_nature_off_site_area.percentage_change, None);

        // Total land: 1000 + 2000 + 300 + 200 = 3500, previous = 3000
        assert_eq!(
            summary.total_use_of_land.reporting_year,
            SquareMeter(3500.0)
        );
        assert_eq!(
            summary.total_use_of_land.percentage_change,
            Some(16.666_666_666_666_664)
        );
    }

    #[test]
    fn it_validates_and_extracts_site_type() {
        // Valid site type
        let mut feature = Feature::default();
        feature.set_property("type", "site");
        assert!(check_property_is_site_type(&feature, "type").is_ok());

        // Extract site type
        let result = geojson_feature_get_site_type(&feature, "type");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), SiteSpecification::Site));

        // Valid nature types
        feature.set_property("type", "natureonsite");
        assert!(check_property_is_site_type(&feature, "type").is_ok());

        feature.set_property("type", "natureoffsite");
        let result = geojson_feature_get_site_type(&feature, "type");
        assert!(matches!(result.unwrap(), SiteSpecification::NatureOffSite));

        // Invalid type
        feature.set_property("type", "invalid");
        let result = check_property_is_site_type(&feature, "type");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid"));

        // Missing property
        feature.remove_property("type");
        assert!(check_property_is_site_type(&feature, "type").is_err());
    }

    #[test]
    fn it_extracts_site_land_use_rows_from_feature() {
        let mut feature = Feature::default();
        feature.set_property("name", "Test Site");
        feature.set_property("type", "site");
        feature.set_property(AREA_COLUMN_NAME, 5000.0);
        feature.set_property(FRACTION_SEALED_COLUMN_NAME, 0.6);

        let result = extract_site_land_use_rows(&feature, "name", "type");
        assert!(result.is_ok());

        let row = result.unwrap();
        assert_eq!(row.location, "Test Site");
        assert!(matches!(row.land_use_type, SiteSpecification::Site));
        assert_eq!(row.area, SquareMeter(5000.0));
        assert_eq!(row.sealed_area, SquareMeter(3000.0)); // 5000 * 0.6

        // Missing area property
        let mut feature_no_area = Feature::default();
        feature_no_area.set_property("name", "Site");
        feature_no_area.set_property("type", "site");
        feature_no_area.set_property(FRACTION_SEALED_COLUMN_NAME, 0.5);
        assert!(extract_site_land_use_rows(&feature_no_area, "name", "type").is_err());

        // Missing fraction sealed
        let mut feature_no_fraction = Feature::default();
        feature_no_fraction.set_property("name", "Site");
        feature_no_fraction.set_property("type", "site");
        feature_no_fraction.set_property(AREA_COLUMN_NAME, 1000.0);
        assert!(extract_site_land_use_rows(&feature_no_fraction, "name", "type").is_err());
    }

    #[test]
    fn it_validates_and_extracts_bbox_from_features() {
        // Test that bbox extraction works with valid features
        // We use json parsing to construct valid geojson rather than manual construction
        let geojson_str = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "id": "1",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]
                },
                "properties": {"name": "Site1", "type": "site"}
            }]
        }"#;

        let geojson: geojson::GeoJson = geojson_str.parse().unwrap();
        let geojson::GeoJson::FeatureCollection(collection) = geojson else {
            panic!("Expected FeatureCollection")
        };

        let collection_input = FeatureCollectionGeoJsonInput {
            value: GeoJsonFeatureCollection::from(collection),
            media_type: crate::processes::parameters::GeoJsonInputMediaType::GeoJson,
        };

        let result = validate_and_extract_bbox(&collection_input, "name", "type");
        assert!(result.is_ok());
        let bbox_result = result.unwrap();
        assert!(bbox_result.errors.is_empty());
    }

    #[test]
    fn it_rejects_invalid_bbox_features() {
        // Test rejection of features without geometry
        let no_geom_str = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": {"name": "NoGeom", "type": "site"}
            }]
        }"#;

        let geojson: geojson::GeoJson = no_geom_str.parse().unwrap();
        let geojson::GeoJson::FeatureCollection(collection) = geojson else {
            panic!("Expected FeatureCollection")
        };

        let collection_input = FeatureCollectionGeoJsonInput {
            value: GeoJsonFeatureCollection::from(collection),
            media_type: crate::processes::parameters::GeoJsonInputMediaType::GeoJson,
        };

        let result = validate_and_extract_bbox(&collection_input, "name", "type");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("All features have errors")
        );

        // Test rejection of invalid geometry types (Point)
        let point_geom_str = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": {"type": "Point", "coordinates": [0, 0]},
                "properties": {"name": "Point", "type": "site"}
            }]
        }"#;

        let geojson: geojson::GeoJson = point_geom_str.parse().unwrap();
        let geojson::GeoJson::FeatureCollection(collection) = geojson else {
            panic!("Expected FeatureCollection")
        };

        let point_input = FeatureCollectionGeoJsonInput {
            value: GeoJsonFeatureCollection::from(collection),
            media_type: crate::processes::parameters::GeoJsonInputMediaType::GeoJson,
        };

        let result = validate_and_extract_bbox(&point_input, "name", "type");
        assert!(result.is_err());

        // Test rejection of invalid site types
        let bad_type_str = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 0]]]},
                "properties": {"name": "BadType", "type": "invalid_type"}
            }]
        }"#;

        let geojson: geojson::GeoJson = bad_type_str.parse().unwrap();
        let geojson::GeoJson::FeatureCollection(collection) = geojson else {
            panic!("Expected FeatureCollection")
        };

        let bad_input = FeatureCollectionGeoJsonInput {
            value: GeoJsonFeatureCollection::from(collection),
            media_type: crate::processes::parameters::GeoJsonInputMediaType::GeoJson,
        };

        let result = validate_and_extract_bbox(&bad_input, "name", "type");
        assert!(result.is_err());
    }

    #[test]
    fn it_builds_sealed_area_vector_operator() {
        let upload_data_id = "test-upload-id".to_string();
        let operators = build_sealed_area_vector_operator(upload_data_id.clone());

        // Verify the operators struct is properly created
        // The sealed_area operator should be a VectorExpression (final step in pipeline)
        match &operators.sealed_area {
            VectorOperator::VectorExpression(_) => {
                // Expected structure for sealed area computation
            }
            _ => panic!("Sealed area operator should be VectorExpression"),
        }

        // The locations_projected operator should be a Reprojection containing OgrSource
        match &operators.locations_projected {
            VectorOperator::Reprojection(_) => {
                // Expected structure for location projection
            }
            _ => panic!("Locations projected operator should be Reprojection"),
        }
    }

    #[test]
    fn it_builds_projected_locations_operator() {
        let upload_data_id = "test-upload-id".to_string();
        let operator = projected_locations_operator(upload_data_id.clone());

        // Verify the operator is a Reprojection type containing OgrSource
        match operator {
            VectorOperator::Reprojection(reprojection) => {
                assert_eq!(
                    reprojection.params.target_spatial_reference,
                    IMPERVIOUSNESS_CRS.as_known_crs(),
                    "Target spatial reference should be imperviousness CRS"
                );
                // The inner source should be an OgrSource
                match &*reprojection.sources.source {
                    SingleRasterOrVectorOperator::VectorOperator(inner) => {
                        match &**inner {
                            VectorOperator::OgrSource(_) => {
                                // Expected structure
                            }
                            _ => panic!("Expected OgrSource as inner operator"),
                        }
                    }
                    SingleRasterOrVectorOperator::RasterOperator(_) => {
                        panic!("Expected VectorOperator source")
                    }
                }
            }
            _ => panic!("Expected Reprojection operator"),
        }
    }

    #[test]
    fn it_constructs_compute_operators_with_correct_pipeline() {
        let upload_data_id = "test-upload-id".to_string();
        let operators = build_sealed_area_vector_operator(upload_data_id);

        // Verify sealed_area operator is VectorExpression with correct configuration
        match &operators.sealed_area {
            VectorOperator::VectorExpression(expr) => {
                let params = &expr.params;
                // Should compute area with correct column name
                assert!(params.expression.contains("area(geom)"));
                assert_eq!(params.geometry_column_name, Some("geom".into()));
            }
            _ => panic!("Expected VectorExpression for sealed area"),
        }

        // Verify locations_projected has correct target CRS
        match &operators.locations_projected {
            VectorOperator::Reprojection(reproj) => {
                assert_eq!(
                    reproj.params.target_spatial_reference,
                    IMPERVIOUSNESS_CRS.as_known_crs()
                );
            }
            _ => panic!("Expected Reprojection for locations"),
        }
    }

    #[tokio::test]
    #[allow(
        clippy::too_many_lines,
        reason = "Test is verbose due to detailed mocking and assertions"
    )]
    async fn it_computes_sealed_surface_correctly() {
        use crate::auth::User;
        use crate::state::USER;
        use geoengine_api_client::models::{
            BoundingBox2D, CollectionType, Coordinate2D, DatasetNameResponse, FeatureDataType,
            GeoJson, IdResponse, Measurement, TypedResultDescriptor, TypedVectorResultDescriptor,
            VectorColumnInfo, VectorDataType,
        };
        use httptest::{
            Expectation, Server,
            matchers::request,
            responders::{cycle, json_encoded},
        };
        use serde_json::json;
        use uuid::Uuid;

        let user = User {
            id: Uuid::from_u128(42),
            session_token: Uuid::from_u128(42).into(),
        };

        USER.scope(user, async {
            // Start httptest server and mock the external Geo Engine endpoints
            let server = Server::run();

            // Mock upload endpoint
            server.expect(
                Expectation::matching(request::method_path("POST", "//upload"))
                    .respond_with(json_encoded(
                        IdResponse::new(
                            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
                        )
                    )),
            );

            // Mock dataset creation endpoint
            server.expect(
                Expectation::matching(request::method_path("POST", "//dataset"))
                    .respond_with(json_encoded(
                        DatasetNameResponse::new(
                            "test-dataset".to_string()
                        )
                    )),
            );

            // Mock workflow registration - 2 calls expected
            server.expect(
                Expectation::matching(request::method_path("POST", "//workflow"))
                .times(2)
                    .respond_with(
                        cycle(vec![
                            Box::new(json_encoded(
                                IdResponse::new(
                                    Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
                                )
                            )),
                            Box::new(json_encoded(
                                IdResponse::new(
                                    Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap()
                                )
                            )),
                        ])
                    ),
            );

            // Mock workflow metadata endpoint for locations_projected
            server.expect(
                Expectation::matching(request::method_path("GET", "//workflow/00000000-0000-0000-0000-000000000003/metadata"))
                    .respond_with(json_encoded(
                        TypedResultDescriptor::Vector(
                            Box::new(TypedVectorResultDescriptor {
                                r#type: Default::default(),
                                data_type: VectorDataType::MultiPolygon,
                                spatial_reference: "EPSG:4326".to_string(),
                                columns: [
                                    ("name".to_string(), VectorColumnInfo {
                                        data_type: FeatureDataType::Text,
                                        measurement: Box::new(Measurement::Unitless(Box::default()))
                                    }),
                                    ("siteType".to_string(), VectorColumnInfo {
                                        data_type: FeatureDataType::Text,
                                        measurement: Box::new(Measurement::Unitless(Box::default()))
                                    })
                                ].into_iter().collect(),
                                bbox: Some(Box::new(BoundingBox2D {
                                    lower_left_coordinate: Coordinate2D::new(8.770, 50.813).into(),
                                    upper_right_coordinate: Coordinate2D::new(8.774, 50.814).into()
                                })),
                                time: None,
                            })
                        )
                    )),
            );

            // Mock WFS endpoint - returns the sealed area computation results
            server.expect(
                Expectation::matching(request::method_path("GET", "//wfs/00000000-0000-0000-0000-000000000002"))
                    .respond_with(json_encoded(GeoJson {
                        r#type: CollectionType::FeatureCollection,
                        features: vec![
                            json!({
                                "type": "Feature",
                                "properties": {
                                    "name": "Site A - Fully Sealed",
                                    "siteType": "site",
                                    "area": 1000.0,
                                    "fractionSealed": 1.0
                                }
                            }),
                            json!({
                                "type": "Feature",
                                "properties": {
                                    "name": "Site B - Partially Sealed",
                                    "siteType": "site",
                                    "area": 2000.0,
                                    "fractionSealed": 0.5
                                }
                            }),
                            json!({
                                "type": "Feature",
                                "properties": {
                                    "name": "Site C - Not Sealed",
                                    "siteType": "site",
                                    "area": 1500.0,
                                    "fractionSealed": 0.0
                                }
                            }),
                            json!({
                                "type": "Feature",
                                "properties": {
                                    "name": "Nature Area - On Site",
                                    "siteType": "natureOnSite",
                                    "area": 500.0,
                                    "fractionSealed": 0.0
                                }
                            }),
                            json!({
                                "type": "Feature",
                                "properties": {
                                    "name": "Nature Area - Off Site",
                                    "siteType": "natureOffSite",
                                    "area": 3000.0,
                                    "fractionSealed": 0.0
                                }
                            }),
                        ],
                    })),
            );

            // Build API configuration pointing to the mock server
            let mut api_config = Configuration::new();
            api_config.base_path = server.url_str("/");

            // Create test input sites
            let sites = serde_json::from_value::<FeatureCollectionGeoJsonInput>(json!({
                "value": {
                    "type": "FeatureCollection",
                    "name": "test-sites",
                    "crs": {
                        "type": "name",
                        "properties": {
                            "name": "urn:ogc:def:crs:OGC:1.3:CRS84"
                        }
                    },
                    "features": [
                        {
                            "type": "Feature",
                            "properties": {
                                "name": "Site A - Fully Sealed",
                                "siteType": "site"
                            },
                            "geometry": {
                                "type": "Polygon",
                                "coordinates": [[[8.773, 50.813], [8.773, 50.8131], [8.7731, 50.8131], [8.7731, 50.813], [8.773, 50.813]]]
                            }
                        },
                        {
                            "type": "Feature",
                            "properties": {
                                "name": "Site B - Partially Sealed",
                                "siteType": "site"
                            },
                            "geometry": {
                                "type": "Polygon",
                                "coordinates": [[[8.771, 50.8125], [8.771, 50.8135], [8.7715, 50.8135], [8.7715, 50.8125], [8.771, 50.8125]]]
                            }
                        },
                        {
                            "type": "Feature",
                            "properties": {
                                "name": "Site C - Not Sealed",
                                "siteType": "site"
                            },
                            "geometry": {
                                "type": "Polygon",
                                "coordinates": [[[8.772, 50.812], [8.772, 50.813], [8.7725, 50.813], [8.7725, 50.812], [8.772, 50.812]]]
                            }
                        },
                        {
                            "type": "Feature",
                            "properties": {
                                "name": "Nature Area - On Site",
                                "siteType": "natureOnSite"
                            },
                            "geometry": {
                                "type": "Polygon",
                                "coordinates": [[[8.770, 50.811], [8.770, 50.8115], [8.7705, 50.8115], [8.7705, 50.811], [8.770, 50.811]]]
                            }
                        },
                        {
                            "type": "Feature",
                            "properties": {
                                "name": "Nature Area - Off Site",
                                "siteType": "natureOffSite"
                            },
                            "geometry": {
                                "type": "Polygon",
                                "coordinates": [[[8.774, 50.814], [8.774, 50.815], [8.7745, 50.815], [8.7745, 50.814], [8.774, 50.814]]]
                            }
                        }
                    ]
                },
                "mediaType": "application/geo+json"
            })).unwrap();

            let year = Year::new(2026);

            // Call compute_site_land_use_data
            let (site_rows, errors) = compute_site_land_use_data(
                &api_config,
                year,
                &sites,
                "name",
                "siteType",
            )
            .await
            .unwrap();

            // Verify no errors in computation
            assert_eq!(errors.len(), 0, "Expected no errors, but got: {errors:?}");

            // Verify we got 5 site rows
            assert_eq!(site_rows.len(), 5, "Expected 5 site rows");

            // Validate sealed area calculations for each site
            // Site A - Fully Sealed: area 1000.0 * fractionSealed 1.0 = 1000.0
            let site_a = &site_rows[0];
            assert_eq!(site_a.location, "Site A - Fully Sealed");
            assert_eq!(site_a.area, SquareMeter(1000.0));
            assert_eq!(site_a.sealed_area, SquareMeter(1000.0), "Site A sealed area should be 1000.0");

            // Site B - Partially Sealed: area 2000.0 * fractionSealed 0.5 = 1000.0
            let site_b = &site_rows[1];
            assert_eq!(site_b.location, "Site B - Partially Sealed");
            assert_eq!(site_b.area, SquareMeter(2000.0));
            assert_eq!(site_b.sealed_area, SquareMeter(1000.0), "Site B sealed area should be 1000.0");

            // Site C - Not Sealed: area 1500.0 * fractionSealed 0.0 = 0.0
            let site_c = &site_rows[2];
            assert_eq!(site_c.location, "Site C - Not Sealed");
            assert_eq!(site_c.area, SquareMeter(1500.0));
            assert_eq!(site_c.sealed_area, SquareMeter(0.0), "Site C sealed area should be 0.0");

            // Nature areas should have no sealed area
            let nature_on_site = &site_rows[3];
            assert_eq!(nature_on_site.location, "Nature Area - On Site");
            assert_eq!(nature_on_site.sealed_area, SquareMeter(0.0));

            let nature_off_site = &site_rows[4];
            assert_eq!(nature_off_site.location, "Nature Area - Off Site");
            assert_eq!(nature_off_site.sealed_area, SquareMeter(0.0));

            // Call compute_summary_from_sites to aggregate data
            let previous_year = TypedPreviousLandUseSummary {
                total_sealed_area: None,
                total_nature_on_site_area: None,
                total_nature_off_site_area: None,
                total_use_of_land: None,
            };

            let summary = compute_summary_from_sites(&site_rows, previous_year);

            // Verify summary calculations
            // Total sealed area: 1000.0 (Site A) + 1000.0 (Site B) + 0.0 (Site C) = 2000.0
            assert_eq!(
                summary.total_sealed_area.reporting_year,
                SquareMeter(2000.0),
                "Total sealed area should be 2000.0"
            );

            // Total nature on-site: 500.0
            assert_eq!(
                summary.total_nature_on_site_area.reporting_year,
                SquareMeter(500.0),
                "Total nature on-site area should be 500.0"
            );

            // Total nature off-site: 3000.0
            assert_eq!(
                summary.total_nature_off_site_area.reporting_year,
                SquareMeter(3000.0),
                "Total nature off-site area should be 3000.0"
            );

            // Total use of land: 1000.0 + 2000.0 + 1500.0 + 500.0 + 3000.0 = 8000.0
            assert_eq!(
                summary.total_use_of_land.reporting_year,
                SquareMeter(8000.0),
                "Total use of land should be 8000.0"
            );

            // No previous year data, so percentage changes should be None
            assert_eq!(summary.total_sealed_area.percentage_change, None);
            assert_eq!(summary.total_nature_on_site_area.percentage_change, None);
            assert_eq!(summary.total_nature_off_site_area.percentage_change, None);
            assert_eq!(summary.total_use_of_land.percentage_change, None);
        }).await;
    }
}
