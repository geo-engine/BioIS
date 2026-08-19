use std::collections::HashMap;

use crate::{
    CONFIG,
    credits::add_credits_used_pending,
    db::DbHandle,
    processes::{
        land_use_sealed_area::{
            compute::{
                compute_available_time_range_from_imperviousness_raster,
                compute_documentation_sources,
            },
            types::{
                LandUseSummaryRowOutput, PreviousLandUseSummary, SiteLandUseRowOutput,
                site_to_data_resource, summary_to_data_resource,
            },
        },
        parameters::{
            DataResource, DataResourceSchema, DocumentationSource, FeatureCollectionGeoJsonInput,
            InputSpec, JsonInput, OutputSpec, RelativeJsonPointer, ToInputHashMap, ToOutputHashMap,
            UnitForArea, Year,
        },
        util::{set_min_max_in_schema, to_output_keys},
    },
    state::{CONTEXT, TaskLocalContext},
    util::{md_content, md_heading},
};
use anyhow::Result;
use compute::{compute_site_land_use_data, compute_summary_from_sites};
use geoengine_api_client::apis::configuration::Configuration;
use ogcapi::{
    processes::Processor,
    types::processes::{
        Execute, ExecuteResult, ExecuteResults, Format, InlineOrRefData, Input, InputValue,
        InputValueNoObject, JobControlOptions, Output, Process, ProcessSummary,
        QualifiedInputValue, TransmissionMode,
        description::{DescriptionType, Metadata},
    },
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod compute;
mod types;

#[doc = include_str!("description.md")]
#[derive(Debug, Clone)]
pub struct LandUseSealedAreaProcess {
    db: DbHandle,
}

impl LandUseSealedAreaProcess {
    pub fn new(db: DbHandle) -> Self {
        Self { db }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LandUseSealedAreaProcessInputs {
    /// `GeoJSON` `FeatureCollection` representing sites to analyze for land-use calculation.
    pub sites: FeatureCollectionGeoJsonInput,

    /// Property name in the features that contains the location/site name.
    pub location_name_field: RelativeJsonPointer,

    /// Property name in the features that contains the site type (e.g., "site", "natureOnSite", "natureOffSite").
    pub site_type_field: RelativeJsonPointer,

    /// Unit for area measurement, with options for hectares (ha) or square meters (m²).
    pub unit_for_area: UnitForArea,

    /// Reporting year for the land-use calculation.
    pub year: Year,

    /// Optional: Site data from the previous reporting period for year-over-year comparison.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_year_data: Option<JsonInput<PreviousLandUseSummary>>,
}

impl TryFrom<&HashMap<String, Input>> for LandUseSealedAreaProcessInputs {
    type Error = anyhow::Error;

    fn try_from(inputs: &HashMap<String, Input>) -> Result<Self, Self::Error> {
        let value = serde_json::to_value(inputs)?;
        let inputs: LandUseSealedAreaProcessInputs = serde_json::from_value(value)?;
        Ok(inputs)
    }
}

mod input_keys {
    pub const SITES: &str = "sites";
    pub const LOCATION_NAME_FIELD: &str = "locationNameField";
    pub const SITE_TYPE_FIELD: &str = "siteTypeField";
    pub const UNIT_FOR_AREA: &str = "unitForArea";
    pub const YEAR: &str = "year";
    pub const PREVIOUS_YEAR_DATA: &str = "previousYearData";
}

#[derive(Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LandUseSealedAreaProcessOutputs {
    /// Land-use summary table with sealed areas, nature-oriented areas, and calculations
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub land_use_summary: Option<DataResource<Vec<LandUseSummaryRowOutput>>>,

    /// Per-site land-use calculations
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub site_land_use_table: Option<DataResource<Vec<SiteLandUseRowOutput>>>,

    /// Echo of inputs for auditing and traceability
    pub inputs: Option<LandUseSealedAreaProcessInputs>,

    /// Errors encountered during processing, if any
    pub errors: Option<Vec<String>>,

    /// Data sources and workflow references used for audits and provenance
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub documentation_sources: Option<DataResource<Vec<DocumentationSource>>>,
}

#[allow(clippy::struct_excessive_bools, reason = "This is not a state machine")]
pub struct OutputKeys {
    pub land_use_summary: bool,
    pub site_land_use_table: bool,
    pub inputs: bool,
    pub errors: bool,
    pub documentation_sources: bool,
}

impl OutputKeys {
    pub const LAND_USE_SUMMARY: &str = "landUseSummary";
    pub const SITE_LAND_USE_TABLE: &str = "siteLandUseTable";
    pub const INPUTS: &str = "inputs";
    pub const ERRORS: &str = "errors";
    pub const DOCUMENTATION_SOURCES: &str = "documentationSources";

    pub fn from_requested_outputs(outputs: &HashMap<String, Output>) -> Result<Self> {
        let outputs = to_output_keys(
            outputs,
            [
                Self::LAND_USE_SUMMARY,
                Self::SITE_LAND_USE_TABLE,
                Self::INPUTS,
                Self::ERRORS,
                Self::DOCUMENTATION_SOURCES,
            ],
        )?;
        Ok(Self {
            land_use_summary: outputs.contains(Self::LAND_USE_SUMMARY),
            site_land_use_table: outputs.contains(Self::SITE_LAND_USE_TABLE),
            inputs: outputs.contains(Self::INPUTS),
            errors: outputs.contains(Self::ERRORS),
            documentation_sources: outputs.contains(Self::DOCUMENTATION_SOURCES),
        })
    }
}

#[async_trait::async_trait]
impl Processor for LandUseSealedAreaProcess {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn version(&self) -> &'static str {
        Self::VERSION
    }

    fn process(&self) -> Result<Process> {
        let configuration = CONTEXT
            .session_token()
            .ok()
            .map(|session_token| CONFIG.geoengine.api_config(Some(session_token)));

        // TODO: make `process` async & get `USER` passed to here
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.process(configuration))
        })
    }

    async fn execute(&self, execute: Execute) -> Result<ExecuteResults> {
        let inputs: LandUseSealedAreaProcessInputs =
            LandUseSealedAreaProcessInputs::try_from(&execute.inputs)?;

        let requested_outputs = OutputKeys::from_requested_outputs(&execute.outputs)?;

        let results = self
            .execute(
                inputs,
                requested_outputs,
                CONFIG.geoengine.api_config(Some(CONTEXT.session_token()?)),
            )
            .await?;

        Ok(results.into())
    }
}

impl LandUseSealedAreaProcess {
    pub const ID: &'static str = "land-use-sealed-area";
    pub const VERSION: &'static str = "0.1.0";

    #[allow(
        clippy::too_many_lines,
        reason = "This function is verbose due to the detailed process description and schema generation."
    )]
    async fn process(&self, configuration: Option<Configuration>) -> Result<Process> {
        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;

        let mut generator = settings.into_generator();

        let mut year_schema = generator.root_schema_for::<Year>();

        if let Some(configuration) = configuration {
            let (start_year, end_year) =
                compute_available_time_range_from_imperviousness_raster(&configuration).await?;
            set_min_max_in_schema(
                &mut year_schema,
                i64::from(start_year.0),
                i64::from(end_year.0),
            )?;
        }

        Ok(Process {
            summary: ProcessSummary {
                id: Self::ID.into(),
                version: Self::VERSION.into(),
                description: DescriptionType {
                    title: Some(md_heading(include_str!("description.md")).to_string()),
                    description: md_content(include_str!("description.md"))
                        .to_string()
                        .into(),
                    keywords: vec![
                        "ESG".to_string(),
                        "biodiversity".to_string(),
                        "land-use".to_string(),
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
            inputs: [
                InputSpec {
                    key: input_keys::SITES,
                    title: "Sites",
                    description: "GeoJSON FeatureCollection of sites to analyze for land-use calculation.",
                    metadata: vec![],
                    r#type: generator.root_schema_for::<FeatureCollectionGeoJsonInput>(),
                },
                InputSpec {
                    key: input_keys::LOCATION_NAME_FIELD,
                    title: "Location Name Field",
                    description: "Reference to the property in the input GeoJSON features that contains the location information.",
                    metadata: vec![Metadata {
                        title: Some("GeoJSON Property Pointer".to_string()),
                        role: Some("json-pointer-base".to_string()),
                        href: Some(
                            "#/inputs/sites/value/features/0/properties".to_string(),
                        ),
                    }],
                    r#type: generator.root_schema_for::<RelativeJsonPointer>(),
                },
                InputSpec {
                    key: input_keys::SITE_TYPE_FIELD,
                    title: "Site Type Field",
                    description: "Reference to the property in the input GeoJSON features that indicates the site type (e.g., 'site', 'natureOnSite', 'natureOffSite').",
                    metadata: vec![Metadata {
                        title: Some("GeoJSON Property Pointer".to_string()),
                        role: Some("json-pointer-base".to_string()),
                        href: Some(
                            "#/inputs/sites/value/features/0/properties".to_string(),
                        ),
                    }],
                    r#type: generator.root_schema_for::<RelativeJsonPointer>(),
                },
                InputSpec {
                    key: input_keys::UNIT_FOR_AREA,
                    title: "Unit for Area",
                    description: "Unit for area measurement, with options for hectares (ha) or square meters (m²).",
                    metadata: vec![],
                    r#type: generator.root_schema_for::<UnitForArea>(),
                },
                InputSpec {
                    key: input_keys::YEAR,
                    title: "Reporting Year",
                    description: "The reporting year for the land-use calculation.",
                    metadata: vec![],
                    r#type: year_schema,
                },
                InputSpec {
                    key: input_keys::PREVIOUS_YEAR_DATA,
                    title: "Previous Year Data (Optional)",
                    description: "GeoJSON FeatureCollection from previous reporting period for comparison.",
                    metadata: vec![],
                    r#type: generator.root_schema_for::<Option<JsonInput<PreviousLandUseSummary>>>(),
                },
            ].into_hash_map(),
            outputs: [
                OutputSpec {
                    key: OutputKeys::LAND_USE_SUMMARY,
                    title: "Land-Use Summary Table",
                    description: "Summary of land-use categories with areas and year-over-year comparison.",
                    r#type: generator.root_schema_for::<DataResource<Vec<LandUseSummaryRowOutput>>>(),
                },
                OutputSpec {
                    key: OutputKeys::SITE_LAND_USE_TABLE,
                    title: "Site Land-Use Table",
                    description: "Per-site land-use calculations with location names and areas.",
                    r#type: generator.root_schema_for::<DataResource<Vec<SiteLandUseRowOutput>>>(),
                },
                OutputSpec {
                    key: OutputKeys::INPUTS,
                    title: "Input Parameters",
                    description: "Echo of inputs for auditing.",
                    r#type: generator.root_schema_for::<LandUseSealedAreaProcessInputs>(),
                },
                OutputSpec {
                    key: OutputKeys::ERRORS,
                    title: "Processing Errors",
                    description: "List of errors encountered during processing, if any.",
                    r#type: generator.root_schema_for::<Vec<String>>(),
                },
                OutputSpec {
                    key: OutputKeys::DOCUMENTATION_SOURCES,
                    title: "Documentation Sources",
                    description: "List of data sources and workflow references used for audits.",
                    r#type: generator.root_schema_for::<DataResource<Vec<DocumentationSource>>>(),
                },
            ].into_hash_map(),
        })
    }

    async fn execute(
        &self,
        inputs: LandUseSealedAreaProcessInputs,
        requested_outputs: OutputKeys,
        configuration: Configuration,
    ) -> Result<LandUseSealedAreaProcessOutputs> {
        let mut outputs = LandUseSealedAreaProcessOutputs {
            land_use_summary: None,
            site_land_use_table: None,
            errors: None,
            inputs: requested_outputs.inputs.then(|| inputs.clone()),
            documentation_sources: None,
        };

        if requested_outputs.land_use_summary
            || requested_outputs.site_land_use_table
            || requested_outputs.errors
        {
            // Compute per-site land-use data
            let (site_rows, errors, computation_id) = compute_site_land_use_data(
                &configuration,
                inputs.year,
                &inputs.sites,
                inputs.location_name_field.as_ref(),
                inputs.site_type_field.as_ref(),
            )
            .await?;

            let land_use_summary = compute_summary_from_sites(
                &site_rows,
                inputs.previous_year_data.as_ref().map(|j| &j.value).into(),
            );

            // Store per-site results if requested
            if requested_outputs.site_land_use_table {
                outputs.site_land_use_table =
                    Some(site_to_data_resource(site_rows, inputs.unit_for_area));
            }

            // Compute summary table from site data
            if requested_outputs.land_use_summary {
                outputs.land_use_summary = Some(summary_to_data_resource(
                    land_use_summary,
                    inputs.unit_for_area,
                ));
            }

            if requested_outputs.errors {
                outputs.errors = Some(errors);
            }

            add_credits_used_pending(self.db.clone(), configuration.clone(), computation_id)
                .await?;
        }

        if requested_outputs.documentation_sources {
            outputs.documentation_sources =
                Some(compute_documentation_sources(&configuration).await?.into());
        }

        Ok(outputs)
    }
}

fn json_format() -> Format {
    Format {
        media_type: Some("application/json".to_string()),
        encoding: None,
        schema: None,
    }
}

impl From<LandUseSealedAreaProcessOutputs> for ExecuteResults {
    fn from(outputs: LandUseSealedAreaProcessOutputs) -> Self {
        let mut result = ExecuteResults::default();

        if let Some(land_use_summary) = outputs.land_use_summary
            && let Ok(value) = land_use_summary.to_input_value()
        {
            result.insert(
                OutputKeys::LAND_USE_SUMMARY.to_string(),
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

        if let Some(site_land_use_table) = outputs.site_land_use_table
            && let Ok(value) = site_land_use_table.to_input_value()
        {
            result.insert(
                OutputKeys::SITE_LAND_USE_TABLE.to_string(),
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
                OutputKeys::INPUTS.to_string(),
                ExecuteResult {
                    output: Output {
                        format: Some(json_format()),
                        transmission_mode: TransmissionMode::Value,
                    },
                    data: InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
                        value: InputValue::Object(
                            inputs_log.as_object().cloned().unwrap_or_default(),
                        ),
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
                OutputKeys::DOCUMENTATION_SOURCES.to_string(),
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
                OutputKeys::ERRORS.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::User, credits::run_lookup_task_once, processes::parameters::GeoJsonInputMediaType,
        state::TaskContext,
    };
    use geoengine_api_client::models::{
        BoundingBox2D, CollectionType, Coordinate2D, DataId, DatasetNameResponse, FeatureDataType,
        GeoJson, IdResponse, InternalDataId, Measurement, OperatorQuota, Provenance,
        ProvenanceEntry, TypedResultDescriptor, TypedVectorResultDescriptor, VectorColumnInfo,
        VectorDataType,
    };
    use geojson::FeatureCollection;
    use httptest::{
        Expectation, Server,
        matchers::request,
        responders::{cycle, json_encoded},
    };
    use ogcapi::types::processes::Input;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::collections::HashMap;
    use uuid::Uuid;

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
                                "location": "Test Site 1",
                                "siteType": "site"
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
                                "location": "Test Site 2",
                                "siteType": "natureOnSite"
                            }
                        },
                        {
                            "type": "Feature",
                            "geometry": {
                                "type": "Point",
                                "coordinates": [8.770_273_718_309_227, 50.807_468_318_244_67]
                            },
                            "id": "missing-location",
                            "properties": {
                                "siteType": "site"
                            }
                        }
                    ]
                },
                "mediaType": "application/geo+json"
            },
            "locationNameField": "location",
            "siteTypeField": "siteType",
            "unitForArea": "ha",
            "year": 2024
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(json).unwrap();
        let json = serde_json::to_value(&inputs).unwrap();

        let _inputs: LandUseSealedAreaProcessInputs = serde_json::from_value(json).unwrap();
    }

    #[test]
    fn it_deserializes_input_with_previous_year_data() {
        let json = json!({
            "sites": {
                "value": {
                    "type": "FeatureCollection",
                    "features": [
                        {
                            "type": "Feature",
                            "geometry": {
                                "type": "Point",
                                "coordinates": [8.770_273_718_309_227, 50.807_468_318_244_67]
                            },
                            "properties": {
                                "location": "Test Site",
                                "siteType": "site"
                            }
                        }
                    ]
                },
                "mediaType": "application/geo+json"
            },
            "year": 2024,
            "locationNameField": "location",
            "siteTypeField": "siteType",
            "unitForArea": "m²",
            "previousYearData": {
                "value": {
                    "totalSealedArea": 1000.0,
                    "totalNatureOnSiteArea": 500.0,
                    "totalNatureOffSiteArea": 200.0,
                    "totalUseOfLand": 1700.0,
                    "unitForArea": "m²"
                },
                "mediaType": "application/json"
            }
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(json).unwrap();
        let json = serde_json::to_value(&inputs).unwrap();

        let deserialized: LandUseSealedAreaProcessInputs = serde_json::from_value(json).unwrap();

        assert_eq!(deserialized.sites.value().features.len(), 1);
        assert!(deserialized.previous_year_data.is_some());
    }

    #[crate::test]
    async fn process_summary_has_expected_inputs_and_outputs(db: DbHandle) {
        let process = LandUseSealedAreaProcess::new(db)
            .process(None)
            .await
            .expect("to produce process description");

        // summary id / version
        assert_eq!(process.summary.id, "land-use-sealed-area");
        assert_eq!(process.summary.version, "0.1.0");

        // job control options contain sync and async execute
        let mut has_sync = false;
        let mut has_async = false;
        for opt in &process.summary.job_control_options {
            match opt {
                JobControlOptions::SyncExecute => has_sync = true,
                JobControlOptions::AsyncExecute => has_async = true,
                JobControlOptions::Dismiss => {}
            }
        }
        assert!(has_sync, "expected SyncExecute in job_control_options");
        assert!(has_async, "expected AsyncExecute in job_control_options");

        // Check required inputs
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

        // Check required outputs
        for key in [
            OutputKeys::LAND_USE_SUMMARY,
            OutputKeys::SITE_LAND_USE_TABLE,
            OutputKeys::INPUTS,
            OutputKeys::ERRORS,
            OutputKeys::DOCUMENTATION_SOURCES,
        ] {
            assert!(
                process.outputs.contains_key(key),
                "expected output key `{key}` in process outputs"
            );
        }
    }

    #[test]
    fn it_handles_geojson_with_multiple_features() {
        let inputs = LandUseSealedAreaProcessInputs {
            year: Year(2024),
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
                        "location": "Test Site 1",
                        "siteType": "site"
                      }
                    },
                    {
                      "type": "Feature",
                      "geometry": {
                        "type": "Point",
                        "coordinates": [8.770_273_718_309_227, 50.807_468_318_244_67]
                      },
                      "properties": {
                        "location": "Test Site 2",
                        "siteType": "natureOnSite"
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
                        "location": "Test Site 3",
                        "siteType": "natureOffSite"
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
            previous_year_data: Default::default(),
        };

        assert_eq!(inputs.sites.value().features.len(), 3);
        assert_eq!(inputs.location_name_field.as_ref(), "location");
        assert_eq!(inputs.site_type_field.as_ref(), "siteType");
    }

    #[test]
    fn it_parses_output_keys_with_different_combinations() {
        let output = Output {
            format: None,
            transmission_mode: Default::default(),
        };

        // Test all outputs requested
        let mut all_outputs = HashMap::new();
        all_outputs.insert(OutputKeys::LAND_USE_SUMMARY.to_string(), output.clone());
        all_outputs.insert(OutputKeys::SITE_LAND_USE_TABLE.to_string(), output.clone());
        all_outputs.insert(OutputKeys::INPUTS.to_string(), output.clone());
        all_outputs.insert(OutputKeys::ERRORS.to_string(), output.clone());
        all_outputs.insert(
            OutputKeys::DOCUMENTATION_SOURCES.to_string(),
            output.clone(),
        );

        let keys = OutputKeys::from_requested_outputs(&all_outputs).unwrap();
        assert!(keys.land_use_summary);
        assert!(keys.site_land_use_table);
        assert!(keys.inputs);
        assert!(keys.errors);
        assert!(keys.documentation_sources);

        // Test subset of outputs
        let mut subset_outputs = HashMap::new();
        subset_outputs.insert(OutputKeys::LAND_USE_SUMMARY.to_string(), output.clone());
        subset_outputs.insert(OutputKeys::ERRORS.to_string(), output.clone());

        let keys = OutputKeys::from_requested_outputs(&subset_outputs).unwrap();
        assert!(keys.land_use_summary);
        assert!(!keys.site_land_use_table);
        assert!(!keys.inputs);
        assert!(keys.errors);
        assert!(!keys.documentation_sources);

        // Test empty outputs defaults to all outputs (per to_output_keys behavior)
        let empty_outputs = HashMap::new();
        let keys = OutputKeys::from_requested_outputs(&empty_outputs).unwrap();
        assert!(keys.land_use_summary);
        assert!(keys.site_land_use_table);
        assert!(keys.inputs);
        assert!(keys.errors);
        assert!(keys.documentation_sources);
    }

    #[test]
    fn it_converts_process_outputs_to_execute_results() {
        // Test with all outputs populated
        let outputs = LandUseSealedAreaProcessOutputs {
            land_use_summary: None,
            site_land_use_table: None,
            inputs: Some(LandUseSealedAreaProcessInputs {
                year: Year(2024),
                sites: FeatureCollectionGeoJsonInput {
                    value: FeatureCollection::default().into(),
                    media_type: GeoJsonInputMediaType::GeoJson,
                },
                location_name_field: "loc".into(),
                site_type_field: "type".into(),
                unit_for_area: UnitForArea::Hectare,
                previous_year_data: None,
            }),
            errors: Some(vec!["error1".to_string(), "error2".to_string()]),
            documentation_sources: None,
        };

        let results: ExecuteResults = outputs.into();

        // Verify inputs and errors are in results
        assert!(results.contains_key(OutputKeys::INPUTS));
        assert!(results.contains_key(OutputKeys::ERRORS));

        // Test with no outputs
        let empty_outputs = LandUseSealedAreaProcessOutputs {
            land_use_summary: None,
            site_land_use_table: None,
            inputs: None,
            errors: None,
            documentation_sources: None,
        };

        let results: ExecuteResults = empty_outputs.into();
        assert!(results.is_empty());
    }

    #[crate::test]
    async fn it_provides_correct_process_metadata(db: DbHandle) {
        let process = LandUseSealedAreaProcess::new(db);
        assert_eq!(process.id(), "land-use-sealed-area");
        assert_eq!(process.version(), "0.1.0");
    }

    #[crate::test(task_context = TaskContext::new(User {
        id: Uuid::from_u128(42),
        session_token: Uuid::from_u128(42).into(),
    }))]
    #[allow(
        clippy::too_many_lines,
        reason = "Test is verbose due to detailed mocking and assertions"
    )]
    #[allow(
        clippy::unreadable_literal,
        clippy::excessive_precision,
        reason = "Ok for coordinates in test data"
    )]
    async fn it_runs_the_process(db: DbHandle) {
        CONTEXT
            .set_job_id(Uuid::from_u128(0x0000_0000_0000_0000_0000))
            .unwrap();

        // Start httptest server and mock the external Geo Engine endpoints
        let server = Server::run();

        server.expect(
            Expectation::matching(request::method_path("POST", "//upload")).respond_with(
                json_encoded(IdResponse::new(
                    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                )),
            ),
        );
        server.expect(
            Expectation::matching(request::method_path("POST", "//dataset")).respond_with(
                json_encoded(DatasetNameResponse::new("test-dataset".to_string())),
            ),
        );
        server.expect(
            Expectation::matching(request::method_path("POST", "//workflow"))
                .times(3)
                .respond_with(cycle(vec![
                    Box::new(json_encoded(IdResponse::new(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                    ))),
                    Box::new(json_encoded(IdResponse::new(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                    ))),
                    Box::new(json_encoded(IdResponse::new(
                        Uuid::parse_str("00000000-0000-0000-0000-000000000004").unwrap(),
                    ))),
                ])),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                "//workflow/00000000-0000-0000-0000-000000000003/metadata",
            ))
            .respond_with(json_encoded(TypedResultDescriptor::Vector(Box::new(
                TypedVectorResultDescriptor {
                    r#type: Default::default(),
                    data_type: VectorDataType::MultiPolygon,
                    spatial_reference: "EPSG:4326".to_string(),
                    columns: [
                        (
                            "name".to_string(),
                            VectorColumnInfo {
                                data_type: FeatureDataType::Text,
                                measurement: Box::new(Measurement::Unitless(Box::default())),
                            },
                        ),
                        (
                            "siteType".to_string(),
                            VectorColumnInfo {
                                data_type: FeatureDataType::Text,
                                measurement: Box::new(Measurement::Unitless(Box::default())),
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    bbox: Some(Box::new(BoundingBox2D {
                        lower_left_coordinate: Coordinate2D::new(8.770, 50.813).into(),
                        upper_right_coordinate: Coordinate2D::new(8.774, 50.813).into(),
                    })),
                    time: None,
                },
            )))),
        );

        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                "//wfs/00000000-0000-0000-0000-000000000002",
            ))
            .respond_with(
                json_encoded(GeoJson {
                    r#type: CollectionType::FeatureCollection,
                    features: vec![
                        json!({
                            "type": "Feature",
                            "properties": {
                                "name": "Musizierhaus",
                                "siteType": "site",
                                "area": 1500.0,
                                "fractionSealed": 1.0
                            }
                        }),
                        json!({
                         "type": "Feature",
                         "properties": {
                             "name": "Im Garten",
                             "siteType": "site",
                             "area": 2000.0,
                             "fractionSealed": 0.75
                         }
                        }),
                        json!({
                         "type": "Feature",
                         "properties": {
                             "name": "Teil der Zentralbib",
                             "siteType": "site",
                             "area": 1800.0,
                             "fractionSealed": 0.8294
                         }
                        }),
                        json!({
                          "type": "Feature",
                          "properties": {
                              "name": "Garten im Musizierhaus",
                              "siteType": "natureOnSite",
                              "area": 500.0,
                              "fractionSealed": 0.0
                          }
                        }),
                        json!({
                            "type": "Feature",
                            "properties": {
                                "name": "Im Alten Botanischen Garten",
                                "siteType": "natureOffSite",
                                "area": 3000.0,
                                "fractionSealed": 0.0
                            }
                        }),
                    ],
                })
                .append_header("x-computation-id", "00000000-0000-0000-0000-000000000003"),
            ),
        );

        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                "//workflow/00000000-0000-0000-0000-000000000004/provenance",
            ))
            .respond_with(json_encoded(vec![ProvenanceEntry {
                provenance: Box::new(Provenance {
                    citation: "CITATION".to_string(),
                    license: "LICENSE".to_string(),
                    uri: "URI".to_string(),
                }),
                data: vec![DataId::Internal(Box::new(InternalDataId {
                    r#type: Default::default(),
                    dataset_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                }))],
            }])),
        );

        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                "//quota/computations/00000000-0000-0000-0000-000000000003",
            ))
            // .times(0..=1) // either background task will have queried it or not, depending on timing
            .respond_with(json_encoded(vec![OperatorQuota::new(
                "OPERATOR-NAME".to_string(),
                "OPERATOR-PATH".to_string(),
                100,
            )])),
        );

        // Build API configuration pointing to the mock server
        let mut api_config = Configuration::new();
        api_config.base_path = server.url_str("/");

        let inputs = serde_json::from_value::<LandUseSealedAreaProcessInputs>(json!({
                "sites": {
                    "value": {
                        "type": "FeatureCollection",
                        "name": "test-data-sealing",
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
                                    "name": "Musizierhaus",
                                    "siteType": "site"
                                },
                                "geometry": {
                                    "type": "Polygon",
                                    "coordinates": [[[8.773084369718358, 50.812878691542942], [8.773108704553691, 50.812363165183598], [8.773636906051513, 50.812368774306606], [8.77363422626267, 50.812881114843449], [8.773084369718358, 50.812878691542942]]]
                                }
                            },
                            {
                                "type": "Feature",
                                "properties": {
                                    "name": "Im Garten",
                                    "siteType": "site"
                                },
                                "geometry": {
                                    "type": "Polygon",
                                    "coordinates": [[[8.771216247876815, 50.812571907069582], [8.771206911327607, 50.812312207328453], [8.771692263238966, 50.812307122122299], [8.771706722708121, 50.8125771236597], [8.771216247876815, 50.812571907069582]]]
                                }
                            },
                            {
                                "type": "Feature",
                                "properties": {
                                    "name": "Teil der Zentralbib",
                                    "siteType": "site"
                                },
                                "geometry": {
                                    "type": "Polygon",
                                    "coordinates": [[[8.771008264934009, 50.813509037898953], [8.771036928561866, 50.813239495110878], [8.771494891149727, 50.813251198211937], [8.771508900259988, 50.813538273899908], [8.771008264934009, 50.813509037898953]]]
                                }
                            },
                            {
                                "type": "Feature",
                                "properties": {
                                    "name": "Garten im Musizierhaus",
                                    "siteType": "natureOnSite"
                                },
                                "geometry": {
                                    "type": "Polygon",
                                    "coordinates": [[[8.773166017675141, 50.812438921387283], [8.773167633417881, 50.812377454260897], [8.773350797293739, 50.812382815303913], [8.773343971472983, 50.812437395514991], [8.773166017675141, 50.812438921387283]]]
                                }
                            },
                            {
                                "type": "Feature",
                                "properties": {
                                    "name": "Im Alten Botanischen Garten",
                                    "siteType": "natureOffSite"
                                },
                                "geometry": {
                                    "type": "Polygon",
                                    "coordinates": [[[8.771981787517911, 50.811958373658889], [8.771975230909757, 50.811797761589858], [8.772320354208373, 50.811794598662132], [8.772310652890782, 50.811958453698445], [8.771981787517911, 50.811958373658889]]]
                                }
                            }
                        ]
                    },
                    "mediaType": "application/geo+json"
                },
                "locationNameField": "name",
                "siteTypeField": "siteType",
                "year": 2026,
                "unitForArea": "m²",
                "previousYearData": {
                    "value": {
                        "totalSealedArea": 1500.0,
                        "totalNatureOnSiteArea": 200.0,
                        "totalNatureOffSiteArea": 800.0,
                        "totalUseOfLand": 2500.0,
                        "unitForArea": "m²"
                    },
                    "mediaType": "application/json"
                }
            })).unwrap();

        let requested_outputs = [
            OutputKeys::LAND_USE_SUMMARY,
            OutputKeys::SITE_LAND_USE_TABLE,
            OutputKeys::INPUTS,
            OutputKeys::ERRORS,
            OutputKeys::DOCUMENTATION_SOURCES,
        ]
        .map(|key| {
            (
                key.to_string(),
                Output {
                    format: None,
                    transmission_mode: Default::default(),
                },
            )
        })
        .into_iter()
        .collect();
        let requested_outputs = OutputKeys::from_requested_outputs(&requested_outputs).unwrap();

        let process = LandUseSealedAreaProcess::new(db.clone());
        let result = process
            .execute(inputs.clone(), requested_outputs, api_config)
            .await
            .unwrap();

        assert_eq!(
            serde_json::to_value(&result).unwrap(),
            json!({
                "landUseSummary": {
                    "name": "Land Use",
                    "data": [
                        {
                            "landUseType": "Total sealed area",
                            "previousYear": 1500.0,
                            "reportingYear": 4492.92,
                            "percentageChange": "199.53 %"
                        },
                        {
                            "landUseType": "Total nature-oriented area on-site",
                            "previousYear": 200.0,
                            "reportingYear": 500.0,
                            "percentageChange": "150 %"
                        },
                        {
                            "landUseType": "Total nature-oriented area off-site",
                            "previousYear": 800.0,
                            "reportingYear": 3000.0,
                            "percentageChange": "275 %"
                        },
                        {
                            "landUseType": "Total use of land",
                            "previousYear": 2500.0,
                            "reportingYear": 8800.0,
                            "percentageChange": "252 %"
                        }
                    ],
                    "schema": {
                        "fields": [
                            {
                                "name": "landUseType",
                                "type": "string",
                                "title": "Land-use type"
                            },
                            {
                                "name": "previousYear",
                                "type": "number",
                                "title": "Previous year (m²)"
                            },
                            {
                                "name": "reportingYear",
                                "type": "number",
                                "title": "Reporting year (m²)"
                            },
                            {
                                "name": "percentageChange",
                                "type": "string",
                                "title": "% change"
                            }
                        ],
                        "primaryKey": [
                            "landUseType"
                        ]
                    }
                },
                "siteLandUseTable": {
                    "name": "Site Land Use",
                    "data": [
                        {
                            "location": "Musizierhaus",
                            "landUseType": "site",
                            "area": 1500.0,
                            "sealedArea": 1500.0
                        },
                        {
                            "location": "Im Garten",
                            "landUseType": "site",
                            "area": 2000.0,
                            "sealedArea": 1500.0
                        },
                        {
                            "location": "Teil der Zentralbib",
                            "landUseType": "site",
                            "area": 1800.0,
                            "sealedArea": 1492.92
                        },
                        {
                            "location": "Garten im Musizierhaus",
                            "landUseType": "natureOnSite",
                            "area": 500.0,
                            "sealedArea": 0.0
                        },
                        {
                            "location": "Im Alten Botanischen Garten",
                            "landUseType": "natureOffSite",
                            "area": 3000.0,
                            "sealedArea": 0.0
                        }
                    ],
                    "schema": {
                        "fields": [
                            {
                                "name": "location",
                                "type": "string",
                                "title": "Location"
                            },
                            {
                                "name": "landUseType",
                                "type": "string",
                                "title": "Land-use type"
                            },
                            {
                                "name": "area",
                                "type": "number",
                                "title": "Area (m²)"
                            },
                            {
                                "name": "sealedArea",
                                "type": "number",
                                "title": "Sealed area (m²)"
                            }
                        ],
                        "primaryKey": [
                            "location"
                        ]
                    }
                },
                "inputs": serde_json::to_value(&inputs).unwrap(),
                "errors": [],
                "documentationSources": {
                    "name": "Documentation Sources",
                    "data": [
                        {
                            "data": "CITATION",
                            "documentation_source": "URI: <a href=\"URI\">URI</a>\nLicense: LICENSE"
                        }
                    ],
                    "schema": {
                        "fields": [
                            {
                                "name": "data",
                                "type": "string",
                                "title": "Data"
                            },
                            {
                                "name": "documentation_source",
                                "type": "string",
                                "title": "Documentation Source"
                            }
                        ],
                        "primaryKey": [
                            "data"
                        ]
                    }
                }
            })
        );

        run_lookup_task_once(db).await.unwrap();
    }
}
