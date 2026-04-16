use anyhow::Result;
use geoengine_api_client::{
    apis::{
        configuration::Configuration, ogcwfs_api::wfs_handler, uploads_api::upload_handler,
        workflows_api::register_workflow_handler,
    },
    models::{
        Aggregation, BandFilter, BandFilterParameters, BandsByNameOrIndex, ColumnNames,
        ContinuousMeasurement, Coordinate2D, Default as ColumnNamesDefault,
        DeriveOutRasterSpecsSource, Expression, ExpressionParameters, FeatureAggregationMethod,
        FirstAggregation, Fraction, GdalSourceParameters, GeoJson, Interpolation,
        InterpolationMethod, InterpolationParameters, InterpolationResolution, Measurement,
        MockPointSource, MockPointSourceParameters, MultiBandGdalSource, RasterBandDescriptor,
        RasterDataType, RasterOperator, RasterStacker, RasterStackerParameters,
        RasterTypeConversion, RasterTypeConversionParameters, RasterVectorJoin,
        RasterVectorJoinParameters, RenameBands, Reprojection, ReprojectionParameters,
        SingleRasterOrVectorOperator, SingleRasterOrVectorSource, SingleRasterSource,
        SingleVectorMultipleRasterSources, SpatialBoundsDerive, SpatialBoundsDeriveNone,
        TemporalAggregationMethod, TemporalRasterAggregation, TemporalRasterAggregationParameters,
        TimeGranularity, TimeStep, VectorOperator, WfsRequest, WfsService,
    },
};
use ogcapi::{
    processes::Processor,
    types::{
        common::Link,
        processes::{
            Execute, ExecuteResult, ExecuteResults, Format, InlineOrRefData, InputValue,
            InputValueNoObject, JobControlOptions, Output, Process, ProcessSummary,
            QualifiedInputValue, TransmissionMode,
            description::{DescriptionType, InputDescription, Metadata, OutputDescription},
        },
    },
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;
use utoipa::ToSchema;

use crate::{
    config::CONFIG,
    processes::parameters::{Coordinate, PointGeoJsonInput},
    state::USER,
    util::{error_response, to_api_workflow},
};

/// Calculates the Normalized Difference Vegetation Index (NDVI) and the corrected NDVI (kNDVI) from satellite imagery.
#[derive(Debug, Clone)]
pub struct NDVIProcess;

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct NDVIProcessInputs {
    pub coordinate: PointGeoJsonInput,
    #[schema(minimum = 2014, maximum = 2014)]
    pub year: Year,
    #[schema(minimum = 1, maximum = 6)]
    pub month: Month,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
pub struct Year(u16);

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
pub struct Month(u16);

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NDVIProcessOutputs {
    pub ndvi: Option<f64>,
    pub k_ndvi: Option<f64>,
    pub inputs: NDVIProcessInputs,
}

impl From<NDVIProcessOutputs> for ExecuteResults {
    fn from(outputs: NDVIProcessOutputs) -> Self {
        let mut result = ExecuteResults::default();

        if let Ok(serde_json::Value::Object(inputs_map)) = serde_json::to_value(&outputs.inputs) {
            result.insert(
                "inputs".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
                        value: InputValue::Object(inputs_map),
                        format: Format {
                            media_type: Some("application/json".to_string()),
                            encoding: Some("utf-8".to_string()),
                            schema: None,
                        },
                    }),
                },
            );
        }

        if let Some(ndvi) = outputs.ndvi {
            result.insert(
                "ndvi".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::Number(ndvi)),
                },
            );
        }
        if let Some(k_ndvi) = outputs.k_ndvi {
            result.insert(
                "kNdvi".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::Number(k_ndvi)),
                },
            );
        }
        result
    }
}

#[async_trait::async_trait]
impl Processor for NDVIProcess {
    fn id(&self) -> &'static str {
        "ndvi"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    #[allow(
        clippy::too_many_lines,
        reason = "description is long but better understood this way"
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
                    // TODO: implement "dismiss extension"
                    // JobControlOptions::Dismiss,
                ],
                output_transmission: vec![TransmissionMode::Value],
                links: vec![
                    Link::new(
                        // TODO: ./ … does not work for some clients
                        format!("./{}/execution", self.id()),
                        "http://www.opengis.net/def/rel/ogc/1.0/execute",
                    )
                    .title("Execution endpoint"),
                ],
            },
            inputs: HashMap::from([(
                "coordinate".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Coordinate in WGS84".to_string()),
                        description: Some(
                            "This is a POINT input in WGS84 (EPSG:4326) format.".to_string(),
                        ),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<PointGeoJsonInput>().to_value(),
                    ..Default::default()
                },
            ),
            ("year".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Year of observation".to_string()),
                        description: Some(
                            "The year for which the NDVI/kNDVI should be calculated.".to_string(),
                        ),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<Year>().to_value(),
                    ..Default::default()
                },
            ),
            ("month".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Month of observation".to_string()),
                        description: Some(
                            "The month (1-12) for which the NDVI/kNDVI should be calculated.".to_string(),
                        ),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<Month>().to_value(),
                    ..Default::default()
                },
            )
            ]),
            outputs: HashMap::from([
                (
                    "inputs".to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some("Input parameters".to_string()),
                            description: Some("The input parameters (coordinate, year, month) for which the NDVI was calculated.".to_string()),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<NDVIProcessInputs>().to_value(),
                    },
                ),
                (
                    "ndvi".to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                "Normalized Difference Vegetation Index (NDVI)".to_string(),
                            ),
                            description: Some(
                                "This is the normalized difference vegetation index (NDVI) value. \
                                The NDVI is calculated using the formula: (NIR - Red) / (NIR + Red), where NIR is the near-infrared band value and Red is the red band value. \
                                The NDVI value ranges from -1 to 1, where higher values indicate healthier vegetation. \
                                Values close to -1 represent water bodies, values around 0 indicate barren areas, and values close to 1 signify dense vegetation. \
                                The NDVI is calculated based on a static MODIS satellite image from 2014." // TODO: Sentinel-2
                                    .into(),
                            ),
                            metadata: vec![Metadata {
                                role: Some("citation".to_string()),
                                title: Some(
                                    "Wikipedia, Normalized Difference Vegetation Index. Accessed on November 13th 2025."
                                    .into()
                                ),
                                href: Some("https://en.wikipedia.org/wiki/Normalized_difference_vegetation_index".to_string()),
                            }],
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<f64>().to_value(),
                    },
                ),
                (
                    "kNdvi".to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                "Kernel Normalized Difference Vegetation Index (kNDVI)".to_string(),
                            ),
                            description: Some(
                                "This is the kernel normalized difference vegetation index (kNDVI) value. \
                                The kNDVI is calculated using the formula: (NIR - Red) / (NIR + Red), where NIR is the near-infrared band value and Red is the red band value. \
                                The kNDVI value ranges from -1 to 1, where higher values indicate healthier vegetation. \
                                Values close to -1 represent water bodies, values around 0 indicate barren areas, and values close to 1 signify dense vegetation. \
                                The kNDVI is calculated based on a static MODIS satellite image from 2014." // TODO: Sentinel-2
                                    .into(),
                            ),
                            metadata: vec![Metadata {
                                role: Some("citation".to_string()),
                                title: Some(
                                    "Gustau Camps-Valls et al., \
                                    A unified vegetation index for quantifying the terrestrial biosphere. \
                                    Sci. Adv.7,eabc7447(2021). DOI:10.1126/sciadv.abc7447"
                                    .into()
                                ),
                                href: Some("https://doi.org/10.1126/sciadv.abc7447".to_string()),
                            }],
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<f64>().to_value(),
                    },
                ),
            ]),
        })
    }

    async fn execute(&self, execute: Execute) -> Result<ExecuteResults> {
        let value = serde_json::to_value(execute.inputs)?;
        let inputs: NDVIProcessInputs = serde_json::from_value(value)?;

        validate_date(inputs.year, inputs.month)?;

        let mut should_compute_ndvi = execute.outputs.is_empty();
        let mut should_compute_k_ndvi = execute.outputs.is_empty();
        for output_key in execute.outputs.keys() {
            match output_key.as_str() {
                "ndvi" => should_compute_ndvi = true,
                "kNdvi" => should_compute_k_ndvi = true,
                other => anyhow::bail!("Unknown output requested: {other}"),
            }
        }

        let outputs = compute_ndvi(
            &CONFIG
                .geoengine
                .api_config(USER.try_get().ok().map(|user| user.session_token)),
            inputs,
            should_compute_ndvi,
            should_compute_k_ndvi,
        )
        .await?;

        Ok(outputs.into())
    }
}

fn validate_date(Year(year): Year, Month(month): Month) -> Result<()> {
    if year != 2020 {
        anyhow::bail!("Year must be 2020");
    }
    if !(1..=12).contains(&month) {
        anyhow::bail!("Month must be between 1 and 12");
    }
    Ok(())
}

#[instrument(skip(configuration), err(Debug))]
async fn compute_ndvi(
    configuration: &Configuration,
    ndvi_process_inputs: NDVIProcessInputs,
    should_compute_ndvi: bool,
    should_compute_k_ndvi: bool,
) -> Result<NDVIProcessOutputs> {
    const NDVI: &str = "NDVI";
    const K_NDVI: &str = "kNDVI";

    // TODO: upload data instead of mocking it
    // let upload_data_id: String = upload_data(&configuration, coordinate)?;
    let vector_source =
        vector_reprojection_source(&ndvi_process_inputs.coordinate.value.coordinates);

    let inputs: Vec<RasterOperator> = match (should_compute_ndvi, should_compute_k_ndvi) {
        (true, true) => vec![ndvi_source(), k_ndvi_source()],
        (true, false) => vec![ndvi_source()],
        (false, true) => vec![k_ndvi_source()],
        (false, false) => {
            return Ok(NDVIProcessOutputs {
                ndvi: None,
                k_ndvi: None,
                inputs: ndvi_process_inputs,
            });
        }
    };
    let workflow = to_api_workflow(&VectorOperator::RasterVectorJoin(
        RasterVectorJoin {
            r#type: Default::default(),
            params: RasterVectorJoinParameters {
                names: ColumnNames::Default(
                    ColumnNamesDefault {
                        r#type: Default::default(),
                    }
                    .into(),
                )
                .into(),
                feature_aggregation: FeatureAggregationMethod::First,
                feature_aggregation_ignore_no_data: Some(false),
                temporal_aggregation: TemporalAggregationMethod::None,
                temporal_aggregation_ignore_no_data: Some(false),
            }
            .into(),
            sources: SingleVectorMultipleRasterSources {
                vector: vector_source.into(),
                rasters: inputs,
            }
            .into(),
        }
        .into(),
    ))?;

    let workflow_id = match register_workflow_handler(configuration, workflow.clone()).await {
        Ok(id) => id,
        Err(e) => {
            let workflow_json = serde_json::to_string_pretty(&workflow)
                .unwrap_or_else(|_| "<failed to serialize workflow>".to_string());
            if let Some(error) = error_response(&e) {
                anyhow::bail!("Failed to register workflow `{workflow_json}`: {error:?}");
            }

            anyhow::bail!("Failed to register workflow `{workflow_json}`: {e}");
        }
    };

    let workflow_id = workflow_id.id.to_string();

    // eprintln!("Registered workflow with ID: {workflow_id}");

    let time_str = format!(
        "{}-{:02}-01T00:00:00Z",
        ndvi_process_inputs.year.0, ndvi_process_inputs.month.0
    );

    // eprintln!("Querying at time: {time_str}");

    tracing::info!(
        coordinate = ?ndvi_process_inputs.coordinate,
        time = time_str,
        workflow_id = workflow_id,
        "Requesting NDVI process"
    );

    // query the whole UTM zone 32N (EPSG:32632), as the result only contains a single coordinate, but the wfs call requires specifying a bbox
    // TODO: create a workflow that works with points of any UTM zone
    let minx = 399_960;
    let miny = 5_590_200;
    let maxx = 509_760;
    let maxy = 5_700_000;

    let feature_collection = wfs_handler(
        configuration,
        &workflow_id,
        WfsRequest::GetFeature,
        Some(&format!("{minx},{miny},{maxx},{maxy}")), // TODO
        None,
        None,
        None,
        None,
        None,
        Some(WfsService::Wfs),
        None,
        Some("EPSG:32632"),
        Some(&time_str),
        Some(&workflow_id),
        None,
    )
    .await?;

    // dbg!(&feature_collection);

    outputs_from_feature_collection(&feature_collection, NDVI, K_NDVI, ndvi_process_inputs)
}

#[allow(unused)] // TODO: implement
async fn upload_data(configuration: &Configuration, coordinate: &Coordinate) -> Result<String> {
    upload_handler(configuration, vec![]).await?;

    anyhow::bail!("Not implemented: upload_data");
}

fn outputs_from_feature_collection(
    feature_collection: &GeoJson,
    ndvi_ref: &str,
    k_ndvi_ref: &str,
    inputs: NDVIProcessInputs,
) -> Result<NDVIProcessOutputs> {
    let mut result = NDVIProcessOutputs {
        ndvi: None,
        k_ndvi: None,
        inputs,
    };

    let Some(first_feature) = feature_collection.features.first() else {
        anyhow::bail!(
            "Input coordinate is outside of the data bounds. Currently, only coordinates in UTM zone 32N (EPSG:32632) are supported (x range 6.0 - 12.0, y range 0.0 - 84.)."
        );
    };

    let Some(properties) = first_feature.get("properties") else {
        anyhow::bail!("No data found for the given coordinate and time.");
    };

    if let Some(features) = properties.get(ndvi_ref)
        && let Some(value) = features.as_f64()
    {
        result.ndvi = Some(value);
    }

    if let Some(features) = properties.get(k_ndvi_ref)
        && let Some(value) = features.as_f64()
    {
        result.k_ndvi = Some(value);
    }

    Ok(result)
}

fn vector_reprojection_source(coordinate: &Coordinate) -> VectorOperator {
    let mock_source = VectorOperator::MockPointSource(
        MockPointSource {
            r#type: Default::default(),
            params: MockPointSourceParameters {
                points: vec![Coordinate2D::new(coordinate.0[0], coordinate.0[1])],
                spatial_bounds: SpatialBoundsDerive::None(
                    SpatialBoundsDeriveNone {
                        r#type: Default::default(),
                    }
                    .into(),
                )
                .into(),
            }
            .into(),
        }
        .into(),
    );

    VectorOperator::Reprojection(
        Reprojection {
            r#type: Default::default(),
            params: ReprojectionParameters {
                derive_out_spec: Some(DeriveOutRasterSpecsSource::ProjectionBounds),
                target_spatial_reference: "EPSG:32632".to_string(),
            }
            .into(),
            sources: SingleRasterOrVectorSource {
                source: SingleRasterOrVectorOperator::VectorOperator(mock_source.into()).into(),
            }
            .into(),
        }
        .into(),
    )
}

fn ndvi_source() -> RasterOperator {
    RasterOperator::TemporalRasterAggregation(
        TemporalRasterAggregation {
            r#type: Default::default(),
            params: TemporalRasterAggregationParameters {
                aggregation: Aggregation::FirstAggregation(
                    FirstAggregation {
                        ignore_no_data: true,
                        r#type: Default::default(),
                    }
                    .into(),
                )
                .into(),
                output_type: None,
                window: TimeStep {
                    granularity: TimeGranularity::Months,
                    step: 1,
                }
                .into(),
                window_reference: None,
            }
            .into(),
            sources: SingleRasterSource {
                raster: ndvi_expression_source("NDVI", ndvi_expression()).into(),
            }
            .into(),
        }
        .into(),
    )
}

fn k_ndvi_source() -> RasterOperator {
    RasterOperator::TemporalRasterAggregation(
        TemporalRasterAggregation {
            r#type: Default::default(),
            params: TemporalRasterAggregationParameters {
                aggregation: Aggregation::FirstAggregation(
                    FirstAggregation {
                        ignore_no_data: true,
                        r#type: Default::default(),
                    }
                    .into(),
                )
                .into(),
                output_type: None,
                window: TimeStep {
                    granularity: TimeGranularity::Months,
                    step: 1,
                }
                .into(),
                window_reference: None,
            }
            .into(),
            sources: SingleRasterSource {
                raster: ndvi_expression_source("kNDVI", k_ndvi_expression()).into(),
            }
            .into(),
        }
        .into(),
    )
}

fn ndvi_expression_source(band_name: &str, expression: String) -> RasterOperator {
    RasterOperator::Expression(
        Expression {
            r#type: Default::default(),
            params: ExpressionParameters {
                expression,
                output_type: RasterDataType::F32,
                output_band: Some(
                    RasterBandDescriptor {
                        name: band_name.to_string(),
                        measurement: Measurement::Continuous(
                            ContinuousMeasurement {
                                measurement: "NDVI".to_string(),
                                r#type: Default::default(),
                                unit: Some(Some("NDVI".to_string())),
                            }
                            .into(),
                        )
                        .into(),
                    }
                    .into(),
                ),
                map_no_data: false,
            }
            .into(),
            sources: SingleRasterSource {
                raster: stac_raster_source().into(),
            }
            .into(),
        }
        .into(),
    )
}

fn stac_raster_source() -> RasterOperator {
    RasterOperator::RasterStacker(
        RasterStacker {
            r#type: Default::default(),
            params: RasterStackerParameters {
                rename_bands: RenameBands::Default(Default::default()).into(),
            }
            .into(),
            sources: geoengine_api_client::models::MultipleRasterSources {
                rasters: vec![scl_source(), nir_red_source()],
            }
            .into(),
        }
        .into(),
    )
}

fn scl_source() -> RasterOperator {
    RasterOperator::RasterTypeConversion(
        RasterTypeConversion {
            r#type: Default::default(),
            params: RasterTypeConversionParameters {
                output_data_type: RasterDataType::U16,
            }
            .into(),
            sources: SingleRasterSource {
                raster: RasterOperator::Interpolation(
                    Interpolation {
                        r#type: Default::default(),
                        params: InterpolationParameters {
                            interpolation: InterpolationMethod::NearestNeighbor,
                            output_origin_reference: None,
                            output_resolution: InterpolationResolution::Fraction(
                                Fraction {
                                    r#type: Default::default(),
                                    x: 2.0,
                                    y: 2.0,
                                }
                                .into(),
                            )
                            .into(),
                        }
                        .into(),
                        sources: SingleRasterSource {
                            raster: RasterOperator::MultiBandGdalSource(
                                MultiBandGdalSource {
                                    r#type: Default::default(),
                                    params: GdalSourceParameters {
                                        data: "sentinel-2-l2a_EPSG32632_U8_20".to_string(),
                                        overview_level: None,
                                    }
                                    .into(),
                                }
                                .into(),
                            )
                            .into(),
                        }
                        .into(),
                    }
                    .into(),
                )
                .into(),
            }
            .into(),
        }
        .into(),
    )
}

fn nir_red_source() -> RasterOperator {
    RasterOperator::BandFilter(
        BandFilter {
            r#type: Default::default(),
            params: BandFilterParameters {
                bands: BandsByNameOrIndex::ArrayVecString(vec!["nir".into(), "red".into()]).into(),
            }
            .into(),
            sources: SingleRasterSource {
                raster: RasterOperator::MultiBandGdalSource(
                    MultiBandGdalSource {
                        r#type: Default::default(),
                        params: GdalSourceParameters {
                            data: "sentinel-2-l2a_EPSG32632_U16_10".to_string(),
                            overview_level: None,
                        }
                        .into(),
                    }
                    .into(),
                )
                .into(),
            }
            .into(),
        }
        .into(),
    )
}

fn ndvi_expression() -> String {
    "if (A == 3 || (A >= 7 && A <= 11)) { NODATA } else { (B - C) / (B + C) }".to_string()
}

fn k_ndvi_expression() -> String {
    indoc::indoc! {
        "if (A == 3 || (A >= 7 && A <= 11)) {
            NODATA
        } else {
            tanh(pow((B - C) / (B + C), 2))
        }"
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use crate::processes::parameters::PointGeoJson;
    use crate::processes::parameters::PointGeoJsonInputMediaType;
    use crate::processes::parameters::PointGeoJsonType;

    use super::*;
    use geoengine_api_client::apis::configuration::Configuration as ApiConfiguration;
    use httptest::matchers::*;
    use httptest::responders::*;
    use httptest::{Expectation, Server};
    use ogcapi::types::processes::Input;
    use serde_json::json;

    #[test]
    fn it_deserializes_the_input() {
        let json = serde_json::json!({
            "coordinate": {
                "value": {
                    "type": "Point",
                    "coordinates": [12.34, 56.78]
                },
                "mediaType": "application/geo+json"
            },
            "year": 2024,
            "month": 3
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(json).unwrap();

        let json = serde_json::to_value(&inputs).unwrap();

        let _inputs: NDVIProcessInputs = serde_json::from_value(json).unwrap();
    }

    #[tokio::test]
    async fn compute_ndvi_integration_with_mock_backend() {
        // Start httptest server and mock the external Geo Engine endpoints
        let server = Server::run();

        // Mock workflow registration (POST /workflow -> { id: "..." })
        server.expect(
            Expectation::matching(request::method("POST")).respond_with(json_encoded(
                json!({ "id": "00000000-0000-0000-0000-000000000003" }),
            )),
        );

        // Mock WFS feature handler (GET /wfs/{workflow} -> GeoJSON with NDVI properties)
        server.expect(
            Expectation::matching(request::method("GET")).respond_with(json_encoded(json!({
                "type": "FeatureCollection",
                "features": [
                    { "type": "Feature", "properties": { "NDVI": 0.123, "kNDVI": 0.456 } }
                ]
            }))),
        );

        // Build API configuration pointing to the mock server
        let mut api_config = ApiConfiguration::new();
        api_config.base_path = server.url_str("");

        // Call compute_ndvi with both outputs requested
        let coord = Coordinate([12.34, 56.78]);

        let outputs = compute_ndvi(
            &api_config,
            NDVIProcessInputs {
                coordinate: PointGeoJsonInput {
                    value: PointGeoJson {
                        r#type: PointGeoJsonType::Point,
                        coordinates: coord,
                    },
                    media_type: PointGeoJsonInputMediaType::GeoJson,
                },
                year: Year(2014),
                month: Month(1),
            },
            true,
            true,
        )
        .await
        .expect("compute_ndvi should succeed");

        assert!(outputs.ndvi.is_some());
        assert!(outputs.k_ndvi.is_some());
        let ndvi = outputs.ndvi.unwrap();
        let k_ndvi = outputs.k_ndvi.unwrap();
        assert!((ndvi - 0.123).abs() < 1e-12);
        assert!((k_ndvi - 0.456).abs() < 1e-12);
    }

    #[test]
    fn process_summary_has_expected_inputs_and_outputs() {
        let p = NDVIProcess;
        let process = p.process().expect("to produce process description");

        // summary id / version
        assert_eq!(process.summary.id, "ndvi");
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

        // inputs contain coordinate, year, month
        assert!(process.inputs.contains_key("coordinate"));
        assert!(process.inputs.contains_key("year"));
        assert!(process.inputs.contains_key("month"));

        // outputs contain ndvi and k_ndvi
        assert!(process.outputs.contains_key("ndvi"));
        assert!(process.outputs.contains_key("kNdvi"));

        // some basic checks for descriptions and schema presence
        let ndvi_output = &process.outputs["ndvi"];
        assert!(ndvi_output.schema.is_object());
    }
}
