use anyhow::{Context, Result};
use geoengine_openapi_client::{
    apis::{
        configuration::Configuration, ogcwfs_api::wfs_handler, uploads_api::upload_handler,
        workflows_api::register_workflow_handler,
    },
    models::{
        ColumnNames, Coordinate2D, Expression, ExpressionParameters, FeatureAggregationMethod,
        GdalSource, GdalSourceParameters, GeoJson, Measurement, MockPointSource,
        MockPointSourceParameters, Names, RasterBandDescriptor, RasterDataType, RasterOperator,
        RasterVectorJoin, RasterVectorJoinParameters, SingleRasterSource,
        SingleVectorMultipleRasterSources, SpatialPartition2D, TemporalAggregationMethod,
        VectorOperator, WfsRequest, WfsService,
    },
};
use ogcapi::{
    processes::Processor,
    types::{
        common::Link,
        processes::{
            Execute, ExecuteResult, ExecuteResults, InlineOrRefData, InputValueNoObject,
            JobControlOptions, Output, Process, ProcessSummary, TransmissionMode,
            description::{DescriptionType, InputDescription, Metadata, OutputDescription},
        },
    },
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{config::CONFIG, state::USER, util::to_api_workflow};

/// Calculates the Normalized Difference Vegetation Index (NDVI) and the corrected NDVI (kNDVI) from satellite imagery.
#[derive(Debug, Clone)]
pub struct NDVIProcess;

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
struct NDVIProcessInputs {
    pub coordinate: PointGeoJsonInput,
    pub year: Year,
    pub month: Month,
}

type Coordinate = [f64; 2];

trait ToBbox {
    fn to_bbox(&self, buffer: f64) -> SpatialPartition2D;
}

impl ToBbox for Coordinate {
    fn to_bbox(&self, buffer: f64) -> SpatialPartition2D {
        use geoengine_openapi_client::models::Coordinate2D;

        let [x, y] = *self;
        SpatialPartition2D::new(
            Coordinate2D::new(x + buffer, y - buffer),
            Coordinate2D::new(x - buffer, y + buffer),
        )
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PointGeoJsonInput {
    pub value: PointGeoJson,
    pub media_type: PointGeoJsonInputMediaType,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum PointGeoJsonInputMediaType {
    #[serde(rename = "application/geo+json")]
    GeoJson,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PointGeoJson {
    pub r#type: PointGeoJsonType,
    pub coordinates: Coordinate,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
enum PointGeoJsonType {
    Point,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, Copy, Clone)]
struct Year(#[schemars(range(min = 2014, max = 2014))] u16);

#[derive(Deserialize, Serialize, Debug, JsonSchema, Copy, Clone)]
struct Month(#[schemars(range(min = 1, max = 6))] u16);

#[derive(Deserialize, Serialize, Debug, JsonSchema)]
struct NDVIProcessOutputs {
    ndvi: Option<f64>,
    k_ndvi: Option<f64>,
}

impl From<NDVIProcessOutputs> for ExecuteResults {
    fn from(outputs: NDVIProcessOutputs) -> Self {
        let mut result = ExecuteResults::default();
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
                "k_ndvi".to_string(),
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
                    "k_ndvi".to_string(),
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
                "k_ndvi" => should_compute_k_ndvi = true,
                other => anyhow::bail!("Unknown output requested: {other}"),
            }
        }

        compute_ndvi(
            &CONFIG
                .geoengine
                .api_config(USER.try_get().ok().map(|user| user.session_token)),
            &inputs.coordinate.value.coordinates,
            inputs.year,
            inputs.month,
            should_compute_ndvi,
            should_compute_k_ndvi,
        )
        .await
        .map(Into::into)
    }
}

fn validate_date(Year(year): Year, Month(month): Month) -> Result<()> {
    if year != 2014 {
        anyhow::bail!("Year must be 2014");
    }
    if !(1..=6).contains(&month) {
        anyhow::bail!("Month must be between 1 and 6");
    }
    Ok(())
}

async fn compute_ndvi(
    configuration: &Configuration,
    coordinate: &Coordinate,
    Year(year): Year,
    Month(month): Month,
    should_compute_ndvi: bool,
    should_compute_k_ndvi: bool,
) -> Result<NDVIProcessOutputs> {
    const NDVI: &str = "NDVI";
    const K_NDVI: &str = "kNDVI";

    // TODO: upload data instead of mocking it
    // let upload_data_id: String = upload_data(&configuration, coordinate)?;
    let vector_source = VectorOperator::MockPointSource(
        MockPointSource {
            r#type: Default::default(),
            params: MockPointSourceParameters {
                points: vec![Coordinate2D::new(coordinate[0], coordinate[1])],
                spatial_bounds: Default::default(),
            }
            .into(),
        }
        .into(),
    );

    let (names, inputs): (Vec<String>, Vec<RasterOperator>) =
        match (should_compute_ndvi, should_compute_k_ndvi) {
            (true, true) => (
                vec![NDVI.into(), K_NDVI.into()],
                vec![ndvi_source(), k_ndvi_source()],
            ),
            (true, false) => (vec![NDVI.into()], vec![ndvi_source()]),
            (false, true) => (vec![K_NDVI.into()], vec![k_ndvi_source()]),
            (false, false) => {
                return Ok(NDVIProcessOutputs {
                    ndvi: None,
                    k_ndvi: None,
                });
            }
        };
    let workflow = to_api_workflow(&VectorOperator::RasterVectorJoin(
        RasterVectorJoin {
            r#type: Default::default(),
            params: RasterVectorJoinParameters {
                names: ColumnNames::Names(Names::new(Default::default(), names).into()).into(),
                feature_aggregation: FeatureAggregationMethod::First,
                feature_aggregation_ignore_no_data: Some(true),
                temporal_aggregation: TemporalAggregationMethod::None,
                temporal_aggregation_ignore_no_data: Some(true),
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

    // eprintln!("{}", serde_json::to_string_pretty(&workflow).unwrap());

    let workflow_id = register_workflow_handler(configuration, workflow).await?;
    let workflow_id = workflow_id.id.to_string();

    // eprintln!("Registered workflow with ID: {workflow_id}");

    let time_str = format!("{year}-{month:02}-01T00:00:00Z");

    // eprintln!("Querying at time: {time_str}");

    tracing::info!(
        coordinate = ?coordinate,
        time = time_str,
        workflow_id = workflow_id,
        "Requesting NDVI process"
    );

    let feature_collection = wfs_handler(
        configuration,
        &workflow_id,
        WfsRequest::GetFeature,
        Some(&coordinate.to_bbox(0.0).to_string()),
        None,
        None,
        None,
        None,
        None,
        Some(WfsService::Wfs),
        None,
        Some("EPSG:4326"),
        Some(&time_str),
        Some(&workflow_id),
        None,
    )
    .await?;

    // dbg!(&feature_collection);

    outputs_from_feature_collection(&feature_collection, NDVI, K_NDVI)
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
) -> Result<NDVIProcessOutputs> {
    let mut result = NDVIProcessOutputs {
        ndvi: None,
        k_ndvi: None,
    };

    let first_feature = feature_collection
        .features
        .first()
        .context("Feature collection is empty")?;

    let properties = first_feature
        .get("properties")
        .context("Feature has no properties")?;

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

fn ndvi_source() -> RasterOperator {
    RasterOperator::Expression(
        Expression {
            r#type: Default::default(),
            params: ExpressionParameters {
                expression: "min((A / (127.50)) - 1, 1)".into(),
                output_type: RasterDataType::F64,
                output_band: Some(
                    RasterBandDescriptor {
                        name: "NDVI".into(),
                        measurement: Measurement::Unitless(Default::default()).into(),
                    }
                    .into(),
                ),
                map_no_data: false,
            }
            .into(),
            sources: SingleRasterSource {
                raster: ndvi_u8_source().into(),
            }
            .into(),
        }
        .into(),
    )
}

fn ndvi_u8_source() -> RasterOperator {
    RasterOperator::GdalSource(
        GdalSource {
            r#type: Default::default(),
            params: GdalSourceParameters {
                data: "ndvi".to_string(),
                overview_level: None,
            }
            .into(),
        }
        .into(),
    )
}

fn k_ndvi_source() -> RasterOperator {
    RasterOperator::Expression(
        Expression {
            r#type: Default::default(),
            params: ExpressionParameters {
                expression: indoc::indoc! {"
                let ndvi = min((A / (127.50)) - 1, 1);
                tanh(pow(ndvi, 2))
            "}
                .into(),
                output_type: RasterDataType::F64,
                output_band: Some(
                    RasterBandDescriptor {
                        name: "kNDVI".into(),
                        measurement: Measurement::Unitless(Default::default()).into(),
                    }
                    .into(),
                ),
                map_no_data: false,
            }
            .into(),
            sources: SingleRasterSource {
                raster: ndvi_u8_source().into(),
            }
            .into(),
        }
        .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use geoengine_openapi_client::apis::configuration::Configuration as ApiConfiguration;
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
        let coord: Coordinate = [12.34, 56.78];

        let outputs = compute_ndvi(&api_config, &coord, Year(2014), Month(1), true, true)
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
        assert!(process.outputs.contains_key("k_ndvi"));

        // some basic checks for descriptions and schema presence
        let ndvi_output = &process.outputs["ndvi"];
        assert!(ndvi_output.schema.is_object());
    }
}
