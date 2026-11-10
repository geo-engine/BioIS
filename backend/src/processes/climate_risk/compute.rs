use crate::db::model::ComputationId;
use crate::profile::CLIMATE_RISK_TABLE_SCHEMA_PROFILE;
use crate::{
    processes::parameters::{
        BioisDisplayKind, BioisDisplayMetadata, BioisTableSchemaExtension, BoundingBox,
        DataResource, DataResourceName, Days, Fields, TableSchemaField, TableSchemaType, Year,
        YearRange,
    },
    util::{error_response, register_processing_graph, to_api_vector_process},
};
use anyhow::Result;
use futures::{TryStreamExt, stream::StreamExt};
use geoengine_api_client::{
    apis::{configuration::Configuration, ogcwfs_api::WfsHandlerError, ogcwfs_api::wfs_handler},
    models::{
        ColumnNames, Coordinate2D, FeatureAggregationMethod, GeoJson, MockPointSource,
        MockPointSourceParameters, Names, RasterVectorJoin, RasterVectorJoinParameters,
        SingleVectorMultipleRasterSources, SpatialBoundsDerive, SpatialBoundsDeriveNone,
        TemporalAggregationMethod, VectorOperator, WfsRequest, WfsService,
    },
};
use geojson::PointType;
use ogcapi::types::processes::{
    ExecuteResult, ExecuteResults, Format, InlineOrRefData, InputValue, Output, QualifiedInputValue,
};
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use tracing::instrument;

use super::ClimateRiskProcess;
use super::types::*;

pub(crate) fn climate_risk_scenario_data_resource(
    scenario: ClimateScenario,
    rows: Vec<ClimateRiskRow>,
    analysis_period: &str,
    reference_period: Option<&str>,
    note: Option<String>,
) -> DataResource<Vec<ClimateRiskRow>> {
    let fields = risk_fields(&rows, reference_period);
    let scenario_name = scenario.properties().name;
    let title = if analysis_period.is_empty() {
        scenario_name.to_string()
    } else {
        format!("{scenario_name} · {analysis_period}")
    };
    let biois = climate_display_extension(&rows);
    DataResource {
        name: DataResourceName::new(format!("climate-risk-{}", scenario.name()))
            .expect("scenario output ids are valid slugs"),
        title: Some(title),
        description: note,
        data: rows,
        schema: Fields {
            fields,
            primary_key: Some(vec!["variable".to_string()]),
            schema: Some(CLIMATE_RISK_TABLE_SCHEMA_PROFILE.to_string()),
            biois: Some(biois),
        },
    }
}

/// Shared column layout for climate-risk tables, excluding any scenario column.
fn risk_fields(rows: &[ClimateRiskRow], reference_period: Option<&str>) -> Vec<TableSchemaField> {
    let mut fields = vec![
        TableSchemaField {
            name: "variable".into(),
            r#type: Some(TableSchemaType::String),
            title: Some("Variable".into()),
            ..Default::default()
        },
        TableSchemaField {
            name: "mean".into(),
            r#type: Some(TableSchemaType::Number),
            title: Some("Mean (days/year)".into()),
            ..Default::default()
        },
        TableSchemaField {
            name: "uncertainty".into(),
            r#type: Some(TableSchemaType::Number),
            title: Some("Uncertainty of the mean (± days/year, 95 %)".into()),
            ..Default::default()
        },
        TableSchemaField {
            name: "meanLabel".into(),
            r#type: Some(TableSchemaType::String),
            title: Some("Mean Label".into()),
            ..Default::default()
        },
        TableSchemaField {
            name: "median".into(),
            r#type: Some(TableSchemaType::Number),
            title: Some("Median (days/year)".into()),
            ..Default::default()
        },
        TableSchemaField {
            name: "min".into(),
            r#type: Some(TableSchemaType::Number),
            title: Some("Min (days/year)".into()),
            ..Default::default()
        },
        TableSchemaField {
            name: "max".into(),
            r#type: Some(TableSchemaType::Number),
            title: Some("Max (days/year)".into()),
            ..Default::default()
        },
        TableSchemaField {
            name: "occurrenceProbability".into(),
            r#type: Some(TableSchemaType::Number),
            title: Some(occurrence_probability_title(reference_period)),
            ..Default::default()
        },
        TableSchemaField {
            name: "riskRatio".into(),
            r#type: Some(TableSchemaType::Number),
            title: Some(risk_ratio_title(reference_period)),
            ..Default::default()
        },
        TableSchemaField {
            name: "occurrenceProbabilityLabel".into(),
            r#type: Some(TableSchemaType::String),
            title: Some("Occurrence Probability Label".into()),
            ..Default::default()
        },
        TableSchemaField {
            name: "occurrenceProbabilityColor".into(),
            r#type: Some(TableSchemaType::String),
            title: Some("Occurrence Probability Color".into()),
            ..Default::default()
        },
    ];
    if rows.iter().any(|row| row.anomaly.is_some()) {
        fields.extend([
            TableSchemaField {
                name: "anomaly".into(),
                r#type: Some(TableSchemaType::Number),
                title: Some(anomaly_title(reference_period)),
                ..Default::default()
            },
            TableSchemaField {
                name: "anomalyLabel".into(),
                r#type: Some(TableSchemaType::String),
                title: Some("Anomaly Label".into()),
                ..Default::default()
            },
            TableSchemaField {
                name: "anomalyColor".into(),
                r#type: Some(TableSchemaType::String),
                title: Some("Anomaly Color".into()),
                ..Default::default()
            },
        ]);
    }
    fields
}

pub(crate) fn raw_ensemble_data_resource(
    mut rows: Vec<ClimateRiskRawRow>,
) -> DataResource<Vec<ClimateRiskRawRow>> {
    rows.sort_by(|a, b| {
        (&a.variable, &a.scenario, &a.model).cmp(&(&b.variable, &b.scenario, &b.model))
    });
    DataResource {
        name: DataResourceName::new("raw-ensemble-data")
            .expect("static resource name is a valid slug"),
        title: Some("Raw Ensemble Data".to_string()),
        description: None,
        data: rows,
        schema: Fields {
            fields: vec![
                TableSchemaField {
                    name: "variable".into(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Variable".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "scenario".into(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Scenario".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "model".into(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Model".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "value".into(),
                    r#type: Some(TableSchemaType::Number),
                    title: Some("Value".into()),
                    ..Default::default()
                },
            ],
            primary_key: Some(vec![
                "variable".to_string(),
                "scenario".to_string(),
                "model".to_string(),
            ]),
            ..Default::default()
        },
    }
}

fn climate_display_extension(rows: &[ClimateRiskRow]) -> BioisTableSchemaExtension {
    let has_anomaly = rows.iter().any(|row| row.anomaly.is_some());
    let mut display = HashMap::from([
        (
            "occurrenceProbability".to_string(),
            BioisDisplayMetadata {
                kind: BioisDisplayKind::RiskProbability,
                label_field: Some("occurrenceProbabilityLabel".to_string()),
                color_field: Some("occurrenceProbabilityColor".to_string()),
            },
        ),
        (
            "mean".to_string(),
            BioisDisplayMetadata {
                kind: BioisDisplayKind::MeanUncertainty,
                label_field: Some("meanLabel".to_string()),
                color_field: None,
            },
        ),
    ]);
    if has_anomaly {
        display.insert(
            "anomaly".to_string(),
            BioisDisplayMetadata {
                kind: BioisDisplayKind::RiskAnomaly,
                label_field: Some("anomalyLabel".to_string()),
                color_field: Some("anomalyColor".to_string()),
            },
        );
    }
    BioisTableSchemaExtension {
        display,
        hidden_fields: [
            "uncertainty",
            "meanLabel",
            "occurrenceProbabilityLabel",
            "occurrenceProbabilityColor",
            "anomalyLabel",
            "anomalyColor",
        ]
        .into_iter()
        // The anomaly label/color columns are hidden only when an anomaly column is present;
        // the uncertainty is shown through the mean's label.
        .filter(|field| has_anomaly || !matches!(*field, "anomalyLabel" | "anomalyColor"))
        .map(str::to_string)
        .collect(),
    }
}

impl TryFrom<ClimateRiskOutputs> for ExecuteResults {
    type Error = anyhow::Error;

    fn try_from(outputs: ClimateRiskOutputs) -> anyhow::Result<Self> {
        let mut result = ExecuteResults::default();

        if let Some(inputs) = outputs.inputs
            && let Some(value) = build_inputs_value(&inputs)
        {
            result.insert("inputs".to_string(), value);
        }

        let mut scenario_notes = outputs.scenario_notes;
        let analysis_period = outputs.analysis_period.as_deref().unwrap_or("");
        let reference_period = outputs.reference_period.as_deref();
        let mut rows_by_scenario: std::collections::BTreeMap<ClimateScenario, Vec<ClimateRiskRow>> =
            std::collections::BTreeMap::new();
        for row in outputs.climate_risk {
            rows_by_scenario
                .entry(row.scenario_id)
                .or_default()
                .push(row);
        }
        for (scenario, rows) in rows_by_scenario {
            match climate_risk_scenario_data_resource(
                scenario,
                rows,
                analysis_period,
                reference_period,
                scenario_notes.remove(&scenario),
            )
            .to_input_value()
            {
                Ok(value) => {
                    result.insert(
                        scenario.name().to_string(),
                        ExecuteResult {
                            output: Output {
                                format: Some(json_format()),
                                transmission_mode: Default::default(),
                            },
                            data: InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
                                value,
                                format: Format {
                                    media_type: Some(
                                        "application/vnd.dataresource+json".to_string(),
                                    ),
                                    encoding: None,
                                    schema: None,
                                },
                            }),
                        },
                    );
                }
                Err(error) => tracing::warn!(
                    "Failed to serialize the climate-risk output for scenario `{}`: {error}",
                    scenario.name()
                ),
            }
        }

        if let Some(raw_ensemble_data) = outputs.raw_ensemble_data {
            match raw_ensemble_data.to_input_value() {
                Ok(value) => {
                    result.insert(
                        "rawEnsembleData".to_string(),
                        ExecuteResult {
                            output: Output {
                                format: Some(json_format()),
                                transmission_mode: Default::default(),
                            },
                            data: InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
                                value,
                                format: Format {
                                    media_type: Some(
                                        "application/vnd.dataresource+json".to_string(),
                                    ),
                                    encoding: None,
                                    schema: None,
                                },
                            }),
                        },
                    );
                }
                Err(error) => {
                    tracing::warn!("Failed to serialize the raw ensemble data output: {error}");
                }
            }
        }

        Ok(result)
    }
}

fn json_format() -> Format {
    Format {
        media_type: Some("application/json".to_string()),
        encoding: Some("utf-8".to_string()),
        schema: None,
    }
}

/// Converts a serialized object into the OGC API's qualified JSON input value.
fn build_qualified_value(object_map: serde_json::Map<String, serde_json::Value>) -> ExecuteResult {
    ExecuteResult {
        output: Output {
            format: None,
            transmission_mode: Default::default(),
        },
        data: InlineOrRefData::QualifiedInputValue(QualifiedInputValue {
            value: InputValue::Object(object_map),
            format: Format {
                media_type: Some("application/json".to_string()),
                encoding: Some("utf-8".to_string()),
                schema: None,
            },
        }),
    }
}

/// Serializes typed inputs at the OGC API boundary, warning and dropping on failure.
fn build_inputs_value(inputs: &ClimateRiskInputs) -> Option<ExecuteResult> {
    let Ok(value) = serde_json::to_value(inputs) else {
        tracing::warn!("Failed to serialize the inputs output");
        return None;
    };
    match value {
        serde_json::Value::Object(object_map) => Some(build_qualified_value(object_map)),
        other => {
            tracing::warn!("Unexpected non-object inputs serialization: {other}");
            None
        }
    }
}
/// One geoengine workflow together with the metadata needed to interpret its results.
struct WorkflowRequest {
    models: Vec<ClimateModelProperties>,
    variable: ClimateVariable,
    scenario: ClimateScenarioProperties,
    workflow: geoengine_api_client::models::ProcessingGraph,
}

/// Builds one workflow per (scenario, variable) pair, using only models that support the scenario.
fn build_workflows(
    coordinate: &PointType,
    variables: &[ClimateVariable],
    scenarios: &[ClimateScenario],
    models: &[ClimateModelProperties],
) -> Vec<WorkflowRequest> {
    scenarios
        .iter()
        .flat_map(|scenario| {
            variables
                .iter()
                .map(|variable| (*variable, scenario.properties()))
        })
        .filter_map(|(variable, scenario_props)| {
            let compatible_models: Vec<ClimateModelProperties> = models
                .iter()
                .filter(|model| model.scenarios.contains(&scenario_props.scenario))
                .cloned()
                .collect();
            if compatible_models.is_empty() {
                return None;
            }

            let raster_sources = compatible_models
                .iter()
                .map(|model| {
                    ClimateRiskProcess::yearly_count_source(variable, model, &scenario_props)
                })
                .collect::<Vec<_>>();
            let model_var_names: Vec<String> = compatible_models
                .iter()
                .map(|model| model.id.clone())
                .collect();

            let workflow = to_api_vector_process(&VectorOperator::RasterVectorJoin(
                RasterVectorJoin {
                    r#type: Default::default(),
                    params: RasterVectorJoinParameters {
                        names: ColumnNames::Names(
                            Names {
                                r#type: Default::default(),
                                values: model_var_names,
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
                        vector: vector_source(coordinate).into(),
                        rasters: raster_sources,
                    }
                    .into(),
                }
                .into(),
            ));
            Some(WorkflowRequest {
                models: compatible_models,
                variable,
                scenario: scenario_props,
                workflow,
            })
        })
        .collect()
}

// bounds geoengine fan-out: a single user request runs one workflow per (variable, scenario)
// plus one historical-reference workflow per variable.
const MAX_CONCURRENT_GEOENGINE_REQUESTS: usize = 8;

/// Runs async jobs with bounded concurrency, yielding results in input order.
async fn run_limited<F, Fut, T, E>(jobs: Vec<F>) -> Result<Vec<T>, E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    // `buffered`: results must stay aligned with the input requests,
    // otherwise a workflow's data is attributed to the wrong (variable, scenario) pair.
    futures::stream::iter(jobs.into_iter().map(|job| job()))
        .buffered(MAX_CONCURRENT_GEOENGINE_REQUESTS)
        .try_collect()
        .await
}

/// Formats a geoengine API error, including the response body when available.
fn unpack_join_error<E>(error: &geoengine_api_client::apis::Error<E>, what: &str) -> anyhow::Error {
    if let Some(response) = error_response(error) {
        anyhow::anyhow!("Failed to {what} `{error}`: {response:?}")
    } else {
        anyhow::anyhow!("Failed to {what} `{error}`")
    }
}

/// Registers a workflow and queries it at the point for the given time range.
async fn run_workflow(
    configuration: &Configuration,
    request: &WorkflowRequest,
    bbox: &str,
    time: &str,
) -> Result<WfsQueryResult> {
    let workflow_id = register_processing_graph(configuration, &request.workflow)
        .await
        .map_err(|error| unpack_join_error(&error, "register a workflow"))?
        .id
        .to_string();
    tracing::debug!(
        "ClimateRisk: registered workflow: variable={}, scenario={}, workflow_id={workflow_id}",
        request.variable.name(),
        request.scenario.scenario.name(),
    );
    wfs_query(configuration, &workflow_id, bbox, time)
        .await
        .map_err(|error| unpack_join_error(&error, "execute a workflow"))
}

/// Means of the analysis and reference values over the models present in both, so the anomaly
/// compares the same ensemble even when a model lacks one of the two scenarios.
#[allow(
    clippy::cast_precision_loss,
    reason = "Averaging over at most a few dozen model values cannot lose relevant precision."
)]
fn paired_means(
    analysis: &HashMap<String, ModelYears>,
    reference: &HashMap<String, ModelYears>,
) -> Option<(f64, f64)> {
    let pairs: Vec<(f64, f64)> = analysis
        .iter()
        .filter_map(|(model, a)| reference.get(model).map(|r| (a.mean, r.mean)))
        .collect();
    if pairs.is_empty() {
        return None;
    }
    let n = pairs.len() as f64;
    Some((
        pairs.iter().map(|(a, _)| a).sum::<f64>() / n,
        pairs.iter().map(|(_, r)| r).sum::<f64>() / n,
    ))
}

/// Aggregates the per-workflow WFS results into climate-risk and raw-ensemble rows.
fn aggregate_rows(
    analysis_results: &[WfsQueryResult],
    reference_results: &[WfsQueryResult],
    workflow_requests: &[WorkflowRequest],
    reference_workflow_requests: &[WorkflowRequest],
) -> Result<(Vec<ClimateRiskRow>, Vec<ClimateRiskRawRow>)> {
    let reference_by_variable: HashMap<ClimateVariable, (&WorkflowRequest, &WfsQueryResult)> =
        reference_workflow_requests
            .iter()
            .zip(reference_results)
            .map(|(request, result)| (request.variable, (request, result)))
            .collect();

    let mut rows = Vec::new();
    let mut raw_rows = Vec::new();
    for (request, analysis) in workflow_requests.iter().zip(analysis_results) {
        let variable_name = request.variable.display_name();
        let model_years = outputs_from_feature_collection(&analysis.geo_json, &request.models)?;
        let Some(aggregated) = aggregate_from_list(&model_years) else {
            continue;
        };
        let reference_years = reference_by_variable
            .get(&request.variable)
            .ok_or_else(|| anyhow::anyhow!("no reference workflow for this variable"))
            .and_then(|(reference_request, result)| {
                outputs_from_feature_collection(&result.geo_json, &reference_request.models)
            })
            .inspect_err(|error| {
                tracing::warn!(
                    "No anomaly and occurrence probability for variable `{}`: {error}",
                    variable_name
                );
            })
            .ok();
        let reference = reference_years
            .as_ref()
            .and_then(|reference_years| paired_means(&model_years, reference_years));
        let counts = reference_years
            .as_ref()
            .and_then(|reference_years| ten_year_event_counts(&model_years, reference_years));
        let occurrence_probability = counts.map(|(analysis, _)| share(analysis));
        let anomaly =
            reference.map(|(analysis_mean, reference_mean)| Days(analysis_mean - reference_mean));
        let pct = reference
            .and_then(|(analysis_mean, reference_mean)| anomaly_pct(analysis_mean, reference_mean));
        let uncertainty = aggregated.standard_error.map(|se| Z_95 * se);
        rows.push(ClimateRiskRow {
            variable: variable_name.to_string(),
            scenario: request.scenario.name.to_string(),
            scenario_id: request.scenario.scenario,
            mean: aggregated.mean,
            median: aggregated.median,
            min: aggregated.min,
            max: aggregated.max,
            occurrence_probability,
            risk_ratio: counts
                .and_then(|(analysis, reference)| risk_ratio(share(analysis), share(reference))),
            anomaly,
            occurrence_probability_label: counts.map(|(analysis, _)| {
                let (low, high) = wilson_interval(analysis);
                probability_range_label(share(analysis), low, high)
            }),
            occurrence_probability_color: occurrence_probability.map(probability_color),
            anomaly_label: anomaly.map(|days| anomaly_label(days.0, pct)),
            anomaly_color: anomaly.map(|days| anomaly_color(days.0, pct)),
            uncertainty: uncertainty.map(Days),
            mean_label: uncertainty.map(|u| format!("{:.1} ± {u:.1}", aggregated.mean.0)),
            standard_error: aggregated.standard_error,
            model_means: aggregated.raw_members.clone().unwrap_or_default(),
        });
        if let Some(raw_members) = aggregated.raw_members {
            for (model_name, value) in raw_members {
                raw_rows.push(ClimateRiskRawRow {
                    variable: variable_name.to_string(),
                    scenario: request.scenario.name.to_string(),
                    model: model_name,
                    value,
                });
            }
        }
    }
    Ok((rows, raw_rows))
}

#[derive(Debug)]
pub(crate) struct ClimatePeriods {
    pub analysis_start: Year,
    pub range: YearRange,
    pub reference_start: Year,
}

fn log_wfs_results(workflow_requests: &[WorkflowRequest], results: &[WfsQueryResult]) {
    for (request, result) in workflow_requests.iter().zip(results) {
        for (j, feature) in result.geo_json.features.iter().enumerate() {
            if let Some(props) = feature.get("properties") {
                tracing::debug!(
                    "ClimateRisk: WFS result: variable={}, scenario={}, feature={}, properties={}",
                    request.variable.name(),
                    request.scenario.scenario.name(),
                    j,
                    props,
                );
            }
        }
    }
}

/// Registers and queries `GeoEngine` workflows for all requested variable/scenario/model
/// combinations plus the historical reference, then aggregates the results into
/// [`ClimateRiskOutputs`].
/// Also returns the geoengine computation ids of all executed WFS queries for credit accounting.
#[instrument(skip(configuration), err(Debug))]
pub(crate) async fn compute_climate(
    configuration: &Configuration,
    coordinate: &PointType,
    periods: &ClimatePeriods,
    variables: &[ClimateVariable],
    scenarios: &[ClimateScenario],
    models: &[ClimateModelProperties],
) -> Result<(ClimateRiskOutputs, Vec<ComputationId>)> {
    let Year(start_year) = periods.analysis_start;
    let YearRange(range) = periods.range;
    let Year(reference_year) = periods.reference_start;
    // ~11 m half-span: small enough to hit a single grid cell, large enough to avoid
    // floating-point edge cases on cell boundaries.
    const POINT_BBOX_HALF_SPAN: f64 = 0.0001;
    if variables.is_empty() || scenarios.is_empty() || models.is_empty() {
        return Ok((ClimateRiskOutputs::default(), Vec::new()));
    }

    let end_analysis = start_year + range;
    let time_str_analysis =
        format!("{start_year:04}-01-01T00:00:00Z/{end_analysis:04}-01-01T00:00:00Z");
    let analysis_period = format!("{start_year:04}–{:04}", end_analysis - 1);
    let end_reference = reference_year + REFERENCE_YEARS;
    let reference_time =
        format!("{reference_year:04}-01-01T00:00:00Z/{end_reference:04}-01-01T00:00:00Z");
    let reference_period = format!("{reference_year:04}–{:04}", end_reference - 1);
    let bbox_string = BoundingBox::around_point(coordinate, POINT_BBOX_HALF_SPAN).wfs_string();

    let analysis_requests = build_workflows(coordinate, variables, scenarios, models);
    // The anomaly baseline comes from the historical tier (1950–2014).
    let reference_requests = build_workflows(
        coordinate,
        variables,
        &[ClimateScenario::Historical],
        models,
    );

    // One job list, so analysis and reference queries run side by side under one limit.
    let jobs = analysis_requests
        .iter()
        .map(|request| (request, time_str_analysis.as_str()))
        .chain(
            reference_requests
                .iter()
                .map(|request| (request, reference_time.as_str())),
        )
        .map(|(request, time)| {
            let bbox = bbox_string.as_str();
            move || run_workflow(configuration, request, bbox, time)
        })
        .collect();
    let mut analysis_results = run_limited(jobs).await?;
    let reference_results = analysis_results.split_off(analysis_requests.len());
    log_wfs_results(&analysis_requests, &analysis_results);
    log_wfs_results(&reference_requests, &reference_results);

    // Every WFS query is a geoengine computation that must be reported for credit accounting.
    let computation_ids: Vec<ComputationId> = analysis_results
        .iter()
        .chain(reference_results.iter())
        .filter_map(|result| result.computation_id)
        .collect();

    let (rows, raw_rows) = aggregate_rows(
        &analysis_results,
        &reference_results,
        &analysis_requests,
        &reference_requests,
    )?;

    let scenario_notes = scenario_notes(&rows);
    Ok((
        ClimateRiskOutputs {
            analysis_period: Some(analysis_period),
            reference_period: Some(reference_period),
            climate_risk: rows,
            scenario_notes,
            raw_ensemble_data: if raw_rows.is_empty() {
                None
            } else {
                Some(raw_ensemble_data_resource(raw_rows))
            },
            inputs: None,
        },
        computation_ids,
    ))
}

/// One WFS query result together with the geoengine computation id reported for it,
/// which is needed to account credit usage.
struct WfsQueryResult {
    geo_json: GeoJson,
    computation_id: Option<ComputationId>,
}

async fn wfs_query(
    configuration: &Configuration,
    workflow_id: &str,
    bbox: &str,
    time: &str,
) -> Result<WfsQueryResult, geoengine_api_client::apis::Error<WfsHandlerError>> {
    let mut computation_id = String::new();
    let geo_json = wfs_handler(
        configuration,
        workflow_id,
        WfsRequest::GetFeature,
        Some(bbox),
        None,
        None,
        None,
        None,
        None,
        Some(WfsService::Wfs),
        None,
        Some("EPSG:4326"),
        Some(time),
        Some(workflow_id),
        None,
        Some(&mut computation_id),
    )
    .await?;
    // Older geoengine instances may not report a computation id; skip credit tracking then.
    let computation_id = ComputationId::from_str(&computation_id).ok();
    Ok(WfsQueryResult {
        geo_json,
        computation_id,
    })
}

/// Builds a single-point vector source for the given coordinate.
pub(crate) fn vector_source(coordinate: &PointType) -> VectorOperator {
    VectorOperator::MockPointSource(
        MockPointSource {
            r#type: Default::default(),
            params: MockPointSourceParameters {
                points: vec![Coordinate2D::new(coordinate[0], coordinate[1])],
                spatial_bounds: Some(Box::new(SpatialBoundsDerive::None(
                    SpatialBoundsDeriveNone {
                        r#type: Default::default(),
                    }
                    .into(),
                ))),
            }
            .into(),
        }
        .into(),
    )
}

/// Computes min, max, mean, and median of the per-model means for a single variable/scenario.
#[allow(
    clippy::cast_precision_loss,
    reason = "Averaging over at most a few dozen model values cannot lose relevant precision."
)]
pub(crate) fn aggregate_from_list(
    model_years: &HashMap<String, ModelYears>,
) -> Option<ClimateVariableResult> {
    if model_years.is_empty() {
        return None;
    }

    let values: Vec<f64> = model_years.values().map(|m| m.mean).collect();
    let min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = values.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let mean = values.iter().sum::<f64>() / values.len() as f64;

    let mut sorted = values.clone();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    let median = if sorted.len().is_multiple_of(2) {
        f64::midpoint(sorted[mid - 1], sorted[mid])
    } else {
        sorted[mid]
    };

    let raw_members = model_years
        .iter()
        .map(|(model, m)| (model.clone(), m.mean))
        .collect();

    // Standard error of the ensemble mean from internal variability, with independent models:
    // sqrt(sum over models of s² / n) / M.
    let standard_error = model_years
        .values()
        .map(|m| {
            m.residual_variance
                .map(|variance| variance / m.values.len() as f64)
        })
        .sum::<Option<f64>>()
        .map(|sum| sum.sqrt() / model_years.len() as f64);

    Some(ClimateVariableResult {
        max: Days(max),
        min: Days(min),
        mean: Days(mean),
        median: Days(median),
        raw_members: Some(raw_members),
        standard_error,
    })
}

/// Return period of the reference event in years. IPCC AR6 WG I reports how the frequency of
/// 10-year events changes (SPM, Figure SPM.6), so the probability classes rate a rare event of
/// today's climate instead of "at least one event day", which is near-certain for most indicators.
const REFERENCE_RETURN_PERIOD: usize = 10;

/// A model's 10-year event level of its reference years: the nearest-rank 90th percentile, so
/// (without ties) one reference year in ten has more event days.
fn ten_year_level(reference: &[f64]) -> Option<f64> {
    let mut sorted = reference.to_vec();
    sorted.sort_by(f64::total_cmp);
    let index = sorted
        .len()
        .checked_sub(sorted.len() / REFERENCE_RETURN_PERIOD + 1)?;
    Some(sorted[index])
}

/// Years above the reference period's 10-year level and all years, as `(events, years)`, in the
/// analysis and in the reference period, over the models present in both. Each model is compared
/// with its own reference level, which removes its bias (as in IPCC AR6 WG I, Figure SPM.6).
// shortcut: empirical quantiles resolve at most a 1-in-(years) level per model, so rarer events
// (classes below "very high") need an extreme-value fit (e.g. GEV) of the reference years.
fn ten_year_event_counts(
    analysis: &HashMap<String, ModelYears>,
    reference: &HashMap<String, ModelYears>,
) -> Option<((usize, usize), (usize, usize))> {
    let mut counts = [(0usize, 0usize); 2]; // (years above the level, years) per period
    for (model, analysis_years) in analysis {
        let Some(reference_years) = reference.get(model) else {
            continue;
        };
        let Some(level) = ten_year_level(&reference_years.values) else {
            continue;
        };
        for (count, values) in counts
            .iter_mut()
            .zip([&analysis_years.values, &reference_years.values])
        {
            count.0 += values.iter().filter(|value| **value > level).count();
            count.1 += values.len();
        }
    }
    let [analysis, reference] = counts;
    (analysis.1 > 0 && reference.1 > 0).then_some((analysis, reference))
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Counting at most a few thousand model-years cannot lose relevant precision."
)]
fn share((events, years): (usize, usize)) -> f64 {
    events as f64 / years as f64
}

/// 95 % Wilson score interval of the share `events / years`. It stays within 0..=1 and holds up
/// for few years and shares near 0 or 1, as with a 5-year window. Model-years count as
/// independent: models are, and the year-to-year correlation of annual counts is small.
#[allow(
    clippy::cast_precision_loss,
    reason = "Counting at most a few thousand model-years cannot lose relevant precision."
)]
fn wilson_interval((events, years): (usize, usize)) -> (f64, f64) {
    let n = years as f64;
    let p = events as f64 / n;
    let z2 = Z_95 * Z_95;
    let center = (p + z2 / (2.0 * n)) / (1.0 + z2 / n);
    let half = Z_95 * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / (1.0 + z2 / n);
    ((center - half).max(0.0), (center + half).min(1.0))
}

/// How many times more often the event occurs than in the reference period (IPCC probability
/// ratio), rounded to one decimal; `None` if it never occurred in the reference period.
fn risk_ratio(analysis: f64, reference: f64) -> Option<f64> {
    (reference > 0.0).then(|| (analysis / reference * 10.0).round() / 10.0)
}

/// Two-sided 95 % quantile of the normal distribution.
const Z_95: f64 = 1.96;
/// Share of models that must agree on the direction of a difference (IPCC AR6 WG I, Atlas).
const MIN_MODEL_AGREEMENT: f64 = 0.8;

/// Variance of the yearly values around their linear trend: the year-to-year variability within
/// the window. Without detrending, a forced trend (e.g. warming within 30 years) would count as
/// noise and inflate the uncertainty.
#[allow(
    clippy::cast_precision_loss,
    reason = "A window has at most a few dozen years."
)]
pub(crate) fn residual_variance(values: &[f64]) -> Option<f64> {
    if values.len() < 3 {
        return None;
    }
    let n = values.len() as f64;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / n;
    let (sxy, sxx) = values
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |(sxy, sxx), (i, y)| {
            let dx = i as f64 - x_mean;
            (sxy + dx * (y - y_mean), sxx + dx * dx)
        });
    let slope = sxy / sxx;
    let residuals = values
        .iter()
        .enumerate()
        .map(|(i, y)| (y - y_mean - slope * (i as f64 - x_mean)).powi(2))
        .sum::<f64>();
    Some(residuals / (n - 2.0))
}

/// Whether two scenarios' means of one variable can be told apart, following the robustness
/// criteria of IPCC AR6 (WG I, Atlas): the difference exceeds the 95 % interval of internal
/// variability, and at least 80 % of the models present in both agree on its direction.
/// Without a standard error there is no basis to call them equal.
#[allow(
    clippy::cast_precision_loss,
    reason = "An ensemble has at most a few dozen models."
)]
fn distinguishable(a: &ClimateRiskRow, b: &ClimateRiskRow) -> bool {
    let (Some(se_a), Some(se_b)) = (a.standard_error, b.standard_error) else {
        return true;
    };
    let difference = a.mean.0 - b.mean.0;
    if difference.abs() <= Z_95 * se_a.hypot(se_b) {
        return false;
    }
    let model_differences: Vec<f64> = a
        .model_means
        .iter()
        .filter_map(|(model, x)| b.model_means.get(model).map(|y| x - y))
        .collect();
    if model_differences.is_empty() {
        return false;
    }
    let agreeing = model_differences
        .iter()
        .filter(|d| d.signum() == difference.signum())
        .count();
    agreeing as f64 / model_differences.len() as f64 >= MIN_MODEL_AGREEMENT
}

/// Per scenario, a note naming the variables whose mean cannot be told apart from another
/// scenario's in this window (see [`distinguishable`]).
pub(crate) fn scenario_notes(rows: &[ClimateRiskRow]) -> BTreeMap<ClimateScenario, String> {
    let short_name = |scenario: ClimateScenario| {
        scenario
            .properties()
            .name
            .split(' ')
            .next()
            .unwrap_or_default()
            .to_string()
    };
    let mut similar: BTreeMap<ClimateScenario, Vec<String>> = BTreeMap::new();
    for row in rows {
        let others: Vec<String> = rows
            .iter()
            .filter(|other| other.variable == row.variable && other.scenario_id != row.scenario_id)
            .filter(|other| !distinguishable(row, other))
            .map(|other| short_name(other.scenario_id))
            .collect();
        if !others.is_empty() {
            similar.entry(row.scenario_id).or_default().push(format!(
                "{} ({})",
                row.variable,
                others.join(", ")
            ));
        }
    }
    similar
        .into_iter()
        .map(|(scenario, variables)| {
            let note = format!(
                "Not distinguishable from other scenarios in this window: {}. A difference \
                 counts when it exceeds the 95 % interval of the year-to-year variability and \
                 at least 80 % of the models agree on its direction.",
                variables.join("; ")
            );
            (scenario, note)
        })
        .collect()
}

const NO_MODEL_COVERAGE_ERROR: &str =
    "Input coordinate not covered by any of the requested climate models for the given time range.";

/// Extracts per-model yearly values from a WFS feature collection response.
#[allow(
    clippy::cast_precision_loss,
    reason = "Climate indicator values are small; i64→f64 and count→f64 cannot lose relevant precision."
)]
pub(crate) fn outputs_from_feature_collection(
    feature_collection: &GeoJson,
    variables: &[ClimateModelProperties],
) -> Result<HashMap<String, ModelYears>> {
    if feature_collection.features.is_empty() {
        anyhow::bail!(NO_MODEL_COVERAGE_ERROR);
    }

    // One feature per time step (year).
    let mut acc: HashMap<String, Vec<f64>> = HashMap::new();
    let mut models_without_data = Vec::new();
    let mut models_with_invalid_data = Vec::new();

    for feature in &feature_collection.features {
        let Some(properties) = feature.get("properties") else {
            continue;
        };

        for model in variables {
            let model_name = &model.id;
            if let Some(value) = properties.get(model_name.as_str()) {
                if let Some(value) = value.as_f64().or_else(|| value.as_i64().map(|v| v as f64)) {
                    acc.entry(model_name.clone()).or_default().push(value);
                } else if !models_with_invalid_data.contains(model_name) {
                    models_with_invalid_data.push(model_name.clone());
                }
            } else if !models_without_data.contains(model_name) {
                models_without_data.push(model_name.clone());
            }
        }
    }

    // Log once per model instead of once per feature × model.
    for model_name in models_without_data {
        tracing::warn!("No data found for model {model_name} in feature properties.");
    }
    for model_name in models_with_invalid_data {
        tracing::warn!("Invalid data type for model {model_name} in feature properties.");
    }

    if acc.is_empty() {
        anyhow::bail!(NO_MODEL_COVERAGE_ERROR);
    }

    Ok(acc
        .into_iter()
        .map(|(model, values)| {
            let years = ModelYears {
                mean: values.iter().sum::<f64>() / values.len() as f64,
                residual_variance: residual_variance(&values),
                values,
            };
            (model, years)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use geoengine_api_client::models::CollectionType;
    use ogcapi::types::processes::InlineOrRefData;

    #[test]
    fn it_groups_result_rows_by_declared_scenario() {
        fn row(scenario: ClimateScenario) -> ClimateRiskRow {
            ClimateRiskRow {
                variable: "Heat Days".to_string(),
                scenario: scenario.properties().name.to_string(),
                scenario_id: scenario,
                mean: Days(1.0),
                median: Days(1.0),
                min: Days(1.0),
                max: Days(1.0),
                occurrence_probability: Some(1.0),
                anomaly: None,
                ..Default::default()
            }
        }
        let outputs = ClimateRiskOutputs {
            inputs: None,
            analysis_period: Some("2041–2070".to_string()),
            reference_period: Some("1981–2010".to_string()),
            climate_risk: vec![
                row(ClimateScenario::Ssp245),
                row(ClimateScenario::Ssp585),
                row(ClimateScenario::Ssp245),
            ],
            raw_ensemble_data: None,
            scenario_notes: Default::default(),
        };

        let result: ExecuteResults = outputs.try_into().unwrap();
        // Keys must be the output ids declared in the process description, not the
        // display names the rows carry.
        assert!(result.contains_key("ssp245"));
        assert!(result.contains_key("ssp585"));
        assert!(!result.contains_key("historical"));
        assert!(!result.contains_key("SSP2-4.5 (Intermediate emissions)"));

        let InlineOrRefData::QualifiedInputValue(qualified) = &result["ssp245"].data else {
            panic!("expected qualified input value");
        };
        let resource: DataResource<Vec<ClimateRiskRow>> =
            serde_json::from_value(serde_json::to_value(&qualified.value).unwrap()).unwrap();
        assert_eq!(resource.data.len(), 2);
        assert_eq!(resource.name.as_str(), "climate-risk-ssp245");
        assert_eq!(
            resource.title.as_deref(),
            Some("SSP2-4.5 (Intermediate emissions) · 2041–2070")
        );
        assert!(
            resource
                .data
                .iter()
                .all(|r| r.scenario == "SSP2-4.5 (Intermediate emissions)")
        );
    }

    fn model_years(mean: f64, years: usize) -> ModelYears {
        ModelYears {
            mean,
            values: vec![mean; years],
            residual_variance: None,
        }
    }

    #[test]
    fn it_aggregate_from_list_aggregates_values() {
        let values = HashMap::from([
            ("MPI-ESM1-2-LR".to_string(), model_years(10.0, 4)),
            ("ACCESS-CM2".to_string(), model_years(20.0, 4)),
        ]);

        let result = aggregate_from_list(&values).unwrap();
        assert_abs_diff_eq!(result.min.0, 10.0);
        assert_abs_diff_eq!(result.max.0, 20.0);
        assert_abs_diff_eq!(result.mean.0, 15.0);
        assert_abs_diff_eq!(result.median.0, 15.0);
        assert_eq!(result.raw_members.as_ref().unwrap().len(), 2);

        assert!(aggregate_from_list(&HashMap::new()).is_none());
    }

    #[test]
    fn it_keeps_the_yearly_values_and_their_variability() {
        let models = vec![ClimateModelProperties {
            id: "ACCESS-CM2".to_string(),
            variant: "r1i1p1f1".to_string(),
            grid: "gn".to_string(),
            scenarios: vec![ClimateScenario::Ssp245],
        }];
        let geo_json = GeoJson {
            features: [0.0, 3.0, 0.0, 1.0]
                .iter()
                .map(|v| serde_json::json!({"type": "Feature", "properties": {"ACCESS-CM2": v}}))
                .collect(),
            r#type: CollectionType::FeatureCollection,
        };

        let years = outputs_from_feature_collection(&geo_json, &models).unwrap();
        // flat trend, residuals -1, 2, -1, 0: variance 6 / (4 - 2)
        assert_eq!(
            years["ACCESS-CM2"],
            ModelYears {
                mean: 1.0,
                values: vec![0.0, 3.0, 0.0, 1.0],
                residual_variance: Some(3.0),
            }
        );
    }

    fn series(values: impl IntoIterator<Item = f64>) -> ModelYears {
        let values: Vec<f64> = values.into_iter().collect();
        ModelYears {
            mean: 0.0,
            values,
            residual_variance: None,
        }
    }

    #[test]
    fn it_finds_the_ten_year_level_of_the_reference_years() {
        let thirty: Vec<f64> = (1..=30).map(f64::from).collect();
        // 3 of 30 years lie above it
        assert_eq!(ten_year_level(&thirty), Some(27.0));
        assert_eq!(ten_year_level(&[5.0, 1.0, 3.0]), Some(5.0));
        assert_eq!(ten_year_level(&[]), None);
    }

    #[test]
    fn it_rates_the_reference_ten_year_event() {
        let reference = HashMap::from([
            ("A".to_string(), series((1..=30).map(f64::from))),
            // a model with a warm bias has its own, higher level
            ("B".to_string(), series((101..=130).map(f64::from))),
        ]);
        let analysis = HashMap::from([
            // 23 years above 27, 7 above 127
            ("A".to_string(), series((21..=50).map(f64::from))),
            (
                "B".to_string(),
                series((101..=130).map(|v| f64::from(v) + 4.0)),
            ),
            // no reference: not rated
            ("C".to_string(), series([1000.0; 30])),
        ]);

        let (events, reference_events) = ten_year_event_counts(&analysis, &reference).unwrap();
        assert_eq!(events, (30, 60));
        assert_eq!(reference_events, (6, 60));
        assert_eq!(
            risk_ratio(share(events), share(reference_events)),
            Some(5.0)
        );
        assert!(probability_label(share(reference_events)).starts_with("9 · "));

        // an event that never happened in the reference period has no ratio
        let reference = HashMap::from([("A".to_string(), series([0.0; 30]))]);
        let analysis = HashMap::from([("A".to_string(), series([0.0, 1.0, 0.0]))]);
        let (events, reference_events) = ten_year_event_counts(&analysis, &reference).unwrap();
        assert_eq!(events, (1, 3));
        assert_eq!(risk_ratio(share(events), share(reference_events)), None);

        assert_eq!(ten_year_event_counts(&analysis, &HashMap::new()), None);
    }

    #[test]
    fn it_widens_the_probability_interval_for_few_years() {
        // 3 of 25 model-years (a 5-year window, 5 models) vs. 18 of 150 (30 years)
        let (low, high) = wilson_interval((3, 25));
        assert_abs_diff_eq!(low, 0.0417, epsilon = 1e-3);
        assert_abs_diff_eq!(high, 0.3, epsilon = 1e-3);
        let (low, high) = wilson_interval((18, 150));
        assert_abs_diff_eq!(low, 0.0774, epsilon = 1e-3);
        assert_abs_diff_eq!(high, 0.1817, epsilon = 1e-3);
        assert_abs_diff_eq!(wilson_interval((0, 25)).0, 0.0);
        assert_abs_diff_eq!(wilson_interval((25, 25)).1, 1.0);
    }

    #[test]
    #[allow(
        clippy::cast_precision_loss,
        reason = "The test series has a handful of values."
    )]
    fn it_detrends_the_yearly_variability() {
        assert_eq!(residual_variance(&[1.0, 2.0]), None);
        assert_abs_diff_eq!(residual_variance(&[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap(), 0.0);

        // adding a trend leaves the variability around it unchanged; the plain variance grows
        let noise = [3.0, -1.0, 4.0, 1.0, -5.0, 9.0, -2.0, 6.0];
        let trended: Vec<f64> = noise
            .iter()
            .enumerate()
            .map(|(i, v)| v + 2.5 * i as f64)
            .collect();
        let plain_variance = |values: &[f64]| {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() as f64 - 1.0)
        };
        assert_abs_diff_eq!(
            residual_variance(&trended).unwrap(),
            residual_variance(&noise).unwrap(),
            epsilon = 1e-9
        );
        assert!(plain_variance(&trended) > 2.0 * residual_variance(&trended).unwrap());
    }

    #[test]
    fn it_combines_the_standard_error_of_independent_models() {
        let model = |mean, residual_variance| ModelYears {
            residual_variance,
            ..model_years(mean, 25)
        };
        let values = HashMap::from([
            ("A".to_string(), model(10.0, Some(4.0))),
            ("B".to_string(), model(20.0, Some(4.0))),
        ]);
        // sqrt(4 / 25 + 4 / 25) / 2
        assert_abs_diff_eq!(
            aggregate_from_list(&values)
                .unwrap()
                .standard_error
                .unwrap(),
            0.32_f64.sqrt() / 2.0,
            epsilon = 1e-12
        );

        let values = HashMap::from([("A".to_string(), model(10.0, None))]);
        assert_eq!(aggregate_from_list(&values).unwrap().standard_error, None);
    }

    fn risk_row(
        variable: &str,
        scenario: ClimateScenario,
        mean: f64,
        standard_error: Option<f64>,
        model_means: &[(&str, f64)],
    ) -> ClimateRiskRow {
        ClimateRiskRow {
            variable: variable.to_string(),
            scenario_id: scenario,
            mean: Days(mean),
            standard_error,
            model_means: model_means
                .iter()
                .map(|(m, v)| (m.to_string(), *v))
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn it_notes_scenarios_that_cannot_be_told_apart() {
        use ClimateScenario::{Ssp245, Ssp585};
        let rows = vec![
            // within the 95 % interval: 0.1 <= 1.96 * hypot(1.5, 1.5)
            risk_row("Heat Days", Ssp245, 14.1, Some(1.5), &[("A", 14.1)]),
            risk_row("Heat Days", Ssp585, 14.0, Some(1.5), &[("A", 14.0)]),
            // far apart and all models agree
            risk_row(
                "Frost Days",
                Ssp245,
                65.0,
                Some(1.0),
                &[("A", 60.0), ("B", 70.0)],
            ),
            risk_row(
                "Frost Days",
                Ssp585,
                32.0,
                Some(1.0),
                &[("A", 30.0), ("B", 34.0)],
            ),
            // far apart, but only 2 of 3 models (67 %) agree on the direction
            risk_row(
                "Tropical Nights",
                Ssp245,
                20.0,
                Some(1.0),
                &[("A", 10.0), ("B", 30.0), ("C", 20.0)],
            ),
            risk_row(
                "Tropical Nights",
                Ssp585,
                30.0,
                Some(1.0),
                &[("A", 40.0), ("B", 20.0), ("C", 30.0)],
            ),
            // no standard error: no basis to call them equal
            risk_row("Ice Days", Ssp245, 18.0, None, &[("A", 18.0)]),
            risk_row("Ice Days", Ssp585, 18.0, None, &[("A", 18.0)]),
        ];

        let notes = scenario_notes(&rows);

        let note = &notes[&Ssp245];
        assert!(note.contains("Heat Days (SSP5-8.5)"), "{note}");
        assert!(note.contains("Tropical Nights (SSP5-8.5)"), "{note}");
        assert!(!note.contains("Frost Days"), "{note}");
        assert!(!note.contains("Ice Days"), "{note}");
        assert!(notes[&Ssp585].contains("Heat Days (SSP2-4.5)"));

        let distinct = vec![
            risk_row("Frost Days", Ssp245, 65.0, Some(1.0), &[("A", 65.0)]),
            risk_row("Frost Days", Ssp585, 32.0, Some(1.0), &[("A", 32.0)]),
        ];
        assert!(scenario_notes(&distinct).is_empty());
    }

    #[test]
    fn it_shows_the_mean_with_its_uncertainty_and_the_note() {
        let rows = vec![ClimateRiskRow {
            variable: "Heat Days".to_string(),
            mean: Days(14.1),
            uncertainty: Some(Days(2.9)),
            mean_label: Some("14.1 ± 2.9".to_string()),
            ..Default::default()
        }];
        let resource = climate_risk_scenario_data_resource(
            ClimateScenario::Ssp245,
            rows,
            "2015–2044",
            None,
            Some(
                "Not distinguishable from other scenarios in this window: Heat Days (SSP5-8.5)."
                    .to_string(),
            ),
        );

        let extension = resource.schema.biois.as_ref().unwrap();
        assert_eq!(
            extension.display["mean"].kind,
            BioisDisplayKind::MeanUncertainty
        );
        assert_eq!(
            extension.display["mean"].label_field.as_deref(),
            Some("meanLabel")
        );
        assert_eq!(extension.display["mean"].color_field, None);
        assert!(extension.hidden_fields.iter().any(|f| f == "meanLabel"));
        assert!(extension.hidden_fields.iter().any(|f| f == "uncertainty"));

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["data"][0]["meanLabel"], "14.1 ± 2.9");
        assert_abs_diff_eq!(json["data"][0]["uncertainty"].as_f64().unwrap(), 2.9);
        assert!(json["data"][0].get("standardError").is_none());
        assert!(
            json["description"]
                .as_str()
                .unwrap()
                .contains("Heat Days (SSP5-8.5)")
        );
    }

    #[test]
    fn it_pairs_models_for_the_anomaly() {
        let analysis = HashMap::from([
            ("A".to_string(), model_years(20.0, 1)),
            ("B".to_string(), model_years(40.0, 1)),
        ]);
        // B has no historical run: comparing all of the analysis to A alone would report +20.
        let reference = HashMap::from([("A".to_string(), model_years(10.0, 1))]);
        assert_eq!(paired_means(&analysis, &reference), Some((20.0, 10.0)));
        assert_eq!(paired_means(&analysis, &HashMap::new()), None);
    }

    #[test]
    fn it_climate_risk_data_resource_declares_display_extension() {
        let rows = vec![ClimateRiskRow {
            variable: "Heat Days".to_string(),
            scenario: "ssp245".to_string(),
            max: Days(100.0),
            min: Days(0.0),
            mean: Days(50.0),
            median: Days(50.0),
            occurrence_probability: Some(0.5),
            anomaly: Some(Days(10.0)),
            ..Default::default()
        }];
        let resource =
            climate_risk_scenario_data_resource(ClimateScenario::Ssp245, rows, "", None, None);
        let probability_field = resource
            .schema
            .fields
            .iter()
            .find(|f| f.name == "occurrenceProbability")
            .unwrap();
        assert!(matches!(
            probability_field.r#type,
            Some(TableSchemaType::Number)
        ));
        assert_eq!(
            resource.schema.schema.as_deref(),
            Some(CLIMATE_RISK_TABLE_SCHEMA_PROFILE)
        );
        assert!(matches!(
            resource.schema.biois.as_ref().unwrap().display["occurrenceProbability"].kind,
            BioisDisplayKind::RiskProbability
        ));
        let probability_metadata =
            &resource.schema.biois.as_ref().unwrap().display["occurrenceProbability"];
        assert_eq!(
            probability_metadata.label_field.as_deref(),
            Some("occurrenceProbabilityLabel")
        );
        assert_eq!(
            probability_metadata.color_field.as_deref(),
            Some("occurrenceProbabilityColor")
        );
    }

    #[test]
    fn it_climate_risk_data_resource_declares_anomaly_display() {
        let rows = vec![
            ClimateRiskRow {
                variable: "Heat Days".to_string(),
                scenario: "ssp245".to_string(),
                max: Days(100.0),
                min: Days(0.0),
                mean: Days(50.0),
                median: Days(50.0),
                occurrence_probability: Some(0.5),
                anomaly: Some(Days(10.0)),
                ..Default::default()
            },
            ClimateRiskRow {
                variable: "Dry Days".to_string(),
                scenario: "ssp245".to_string(),
                max: Days(100.0),
                min: Days(0.0),
                mean: Days(50.0),
                median: Days(50.0),
                occurrence_probability: Some(0.5),
                anomaly: Some(Days(5.0)),
                ..Default::default()
            },
        ];
        let resource =
            climate_risk_scenario_data_resource(ClimateScenario::Ssp245, rows, "", None, None);

        let field_names: std::collections::HashSet<&str> = resource
            .schema
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        let extension = resource.schema.biois.as_ref().unwrap();
        for metadata in extension.display.values() {
            assert!(field_names.contains(metadata.label_field.as_deref().unwrap()));
            // the mean's label carries no color
            if let Some(color_field) = metadata.color_field.as_deref() {
                assert!(field_names.contains(color_field));
            }
        }
        assert!(
            extension
                .hidden_fields
                .iter()
                .all(|field| field_names.contains(field.as_str()))
        );

        assert!(matches!(
            resource.schema.biois.as_ref().unwrap().display["anomaly"].kind,
            BioisDisplayKind::RiskAnomaly
        ));
        let anomaly_metadata = &resource.schema.biois.as_ref().unwrap().display["anomaly"];
        assert_eq!(
            anomaly_metadata.label_field.as_deref(),
            Some("anomalyLabel")
        );
        assert_eq!(
            anomaly_metadata.color_field.as_deref(),
            Some("anomalyColor")
        );
    }

    #[test]
    fn it_climate_risk_data_resource_omits_anomaly_field_when_absent() {
        let rows = vec![ClimateRiskRow {
            variable: "Heat Days".to_string(),
            scenario: "ssp245".to_string(),
            max: Days(100.0),
            min: Days(0.0),
            mean: Days(50.0),
            median: Days(50.0),
            occurrence_probability: Some(0.5),
            anomaly: None,
            ..Default::default()
        }];
        let resource =
            climate_risk_scenario_data_resource(ClimateScenario::Ssp245, rows, "", None, None);
        let field_names: Vec<&str> = resource
            .schema
            .fields
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert!(!field_names.contains(&"anomaly"));
        assert!(field_names.contains(&"occurrenceProbability"));
        assert!(field_names.contains(&"occurrenceProbabilityLabel"));
        assert!(field_names.contains(&"occurrenceProbabilityColor"));
        assert!(
            !resource
                .schema
                .biois
                .as_ref()
                .unwrap()
                .display
                .contains_key("anomaly")
        );
    }

    #[test]
    fn it_climate_risk_scenario_data_resource_annotates_periods() {
        let rows = vec![ClimateRiskRow {
            variable: "Heat Days".to_string(),
            scenario: "SSP2-4.5 (Intermediate emissions)".to_string(),
            max: Days(100.0),
            min: Days(0.0),
            mean: Days(50.0),
            median: Days(50.0),
            occurrence_probability: Some(0.5),
            anomaly: Some(Days(10.0)),
            ..Default::default()
        }];
        let resource = climate_risk_scenario_data_resource(
            ClimateScenario::Ssp245,
            rows,
            "2041–2070",
            Some("1981–2010"),
            None,
        );

        assert_eq!(resource.name.as_str(), "climate-risk-ssp245");
        assert_eq!(
            resource.title.as_deref(),
            Some("SSP2-4.5 (Intermediate emissions) · 2041–2070")
        );
        let title = |name: &str| {
            resource
                .schema
                .fields
                .iter()
                .find(|f| f.name == name)
                .unwrap()
                .title
                .clone()
        };
        assert_eq!(title("mean").as_deref(), Some("Mean (days/year)"));
        assert_eq!(title("median").as_deref(), Some("Median (days/year)"));
        assert_eq!(
            title("anomaly").as_deref(),
            Some("Anomaly (days/year compared to 1981–2010)")
        );

        let resource = climate_risk_scenario_data_resource(
            ClimateScenario::Ssp245,
            vec![ClimateRiskRow {
                variable: "Heat Days".to_string(),
                scenario: "SSP2-4.5 (Intermediate emissions)".to_string(),
                max: Days(100.0),
                min: Days(0.0),
                mean: Days(50.0),
                median: Days(50.0),
                occurrence_probability: Some(0.5),
                anomaly: None,
                ..Default::default()
            }],
            "",
            None,
            None,
        );
        assert_eq!(resource.name.as_str(), "climate-risk-ssp245");
        assert_eq!(
            resource.title.as_deref(),
            Some("SSP2-4.5 (Intermediate emissions)")
        );
        assert!(resource.schema.fields.iter().all(|f| f.name != "anomaly"));
    }

    #[test]
    fn it_converts_outputs_into_execute_results() {
        let rows = vec![ClimateRiskRow {
            variable: "Heat Days".to_string(),
            scenario: "ssp245".to_string(),
            max: Days(100.0),
            min: Days(0.0),
            mean: Days(50.0),
            median: Days(50.0),
            scenario_id: ClimateScenario::Ssp245,
            occurrence_probability: None,
            anomaly: Some(Days(10.0)),
            ..Default::default()
        }];
        let raw_rows = vec![ClimateRiskRawRow {
            variable: "Heat Days".to_string(),
            scenario: "ssp245".to_string(),
            model: "ACCESS-CM2".to_string(),
            value: 42.0,
        }];
        let outputs = ClimateRiskOutputs {
            inputs: None,
            analysis_period: None,
            reference_period: None,
            climate_risk: rows,
            raw_ensemble_data: Some(raw_ensemble_data_resource(raw_rows)),
            scenario_notes: Default::default(),
        };
        let results: ExecuteResults = outputs.try_into().unwrap();
        assert!(results.contains_key("ssp245"));
        assert!(!results.contains_key("ssp585"));
        assert!(results.contains_key("rawEnsembleData"));
    }

    #[test]
    fn it_averages_per_model_values_from_feature_collection() {
        let models = vec![
            ClimateModelProperties {
                id: "MPI-ESM1-2-LR".to_string(),
                variant: "r1i1p1f1".to_string(),
                grid: "gn".to_string(),
                scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
            },
            ClimateModelProperties {
                id: "ACCESS-CM2".to_string(),
                variant: "r1i1p1f1".to_string(),
                grid: "gn".to_string(),
                scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
            },
        ];

        let geo_json = GeoJson {
            features: vec![
                serde_json::json!({"type": "Feature", "properties": {"MPI-ESM1-2-LR": 42.0, "ACCESS-CM2": 10.0}}),
                serde_json::json!({"type": "Feature", "properties": {"MPI-ESM1-2-LR": 58.0, "ACCESS-CM2": 30.0}}),
            ],
            r#type: CollectionType::FeatureCollection,
        };
        let result = outputs_from_feature_collection(&geo_json, &models).unwrap();
        assert_eq!(result.len(), 2);
        assert_abs_diff_eq!(result["MPI-ESM1-2-LR"].mean, 50.0);
        assert_eq!(result["MPI-ESM1-2-LR"].values, [42.0, 58.0]);
        assert_abs_diff_eq!(result["ACCESS-CM2"].mean, 20.0);
        assert_eq!(result["ACCESS-CM2"].values, [10.0, 30.0]);

        assert!(outputs_from_feature_collection(&GeoJson::default(), &models).is_err());

        let geo_json = GeoJson {
            features: vec![serde_json::json!({"type": "Feature"})],
            r#type: CollectionType::FeatureCollection,
        };
        assert!(outputs_from_feature_collection(&geo_json, &models).is_err());
    }

    #[test]
    fn it_vector_source_creates_mock_point_source() {
        let point = PointType::from(vec![12.0, 34.0]);
        let result = vector_source(&point);
        assert!(matches!(result, VectorOperator::MockPointSource(_)));
    }

    #[test]
    fn it_sorts_raw_ensemble_rows_by_variable_scenario_model() {
        let rows = vec![
            ClimateRiskRawRow {
                variable: "Ice Days".to_string(),
                scenario: "ssp245".to_string(),
                model: "ACCESS-CM2".to_string(),
                value: 1.0,
            },
            ClimateRiskRawRow {
                variable: "Heat Days".to_string(),
                scenario: "ssp585".to_string(),
                model: "MPI-ESM1-2-LR".to_string(),
                value: 2.0,
            },
            ClimateRiskRawRow {
                variable: "Heat Days".to_string(),
                scenario: "historical".to_string(),
                model: "ACCESS-CM2".to_string(),
                value: 3.0,
            },
        ];

        let resource = raw_ensemble_data_resource(rows);

        let keys: Vec<_> = resource
            .data
            .iter()
            .map(|r| (r.variable.as_str(), r.scenario.as_str(), r.model.as_str()))
            .collect();
        assert_eq!(
            keys,
            vec![
                ("Heat Days", "historical", "ACCESS-CM2"),
                ("Heat Days", "ssp585", "MPI-ESM1-2-LR"),
                ("Ice Days", "ssp245", "ACCESS-CM2"),
            ]
        );
    }

    #[test]
    fn it_embeds_serialized_inputs_in_execute_results() {
        let payload = serde_json::json!({
            "coordinate": {
                "value": {"type": "Point", "coordinates": [12.34, 56.78]},
                "mediaType": "application/geo+json"
            },
            "yearBegin": 2014,
            "yearRange": 20,
            "referenceYearBegin": 2020,
        });
        let inputs: ClimateRiskInputs = serde_json::from_value(payload).unwrap();
        let outputs = ClimateRiskOutputs {
            inputs: Some(inputs),
            analysis_period: None,
            reference_period: None,
            climate_risk: Vec::new(),
            raw_ensemble_data: None,
            scenario_notes: Default::default(),
        };

        let results: ExecuteResults = outputs.try_into().unwrap();

        assert!(results.contains_key("inputs"));
        let InlineOrRefData::QualifiedInputValue(qualified) = &results["inputs"].data else {
            panic!("expected qualified input value");
        };
        let value =
            serde_json::to_value(&qualified.value).expect("inputs value must be serializable");
        assert_eq!(value["yearBegin"], 2014);
        assert_eq!(value["coordinate"]["mediaType"], "application/geo+json");
    }

    #[test]
    fn it_filters_models_per_scenario_when_building_workflows() {
        let point = PointType::from(vec![8.0, 50.0]);
        // Only supports SSP245, so SSP585 requests must be filtered out.
        let models = vec![ClimateModelProperties {
            id: "LIMITED-MODEL".to_string(),
            variant: "r1i1p1f1".to_string(),
            grid: "gn".to_string(),
            scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
        }];
        let workflows = build_workflows(
            &point,
            &[ClimateVariable::HeatDays, ClimateVariable::IceDays],
            &[ClimateScenario::Ssp245, ClimateScenario::Ssp585],
            &models,
        );

        assert_eq!(workflows.len(), 2);
        assert!(
            workflows
                .iter()
                .all(|w| w.scenario.scenario == ClimateScenario::Ssp245)
        );
        assert_eq!(workflows[0].variable, ClimateVariable::HeatDays);
        assert_eq!(workflows[1].variable, ClimateVariable::IceDays);
        assert_eq!(workflows[0].models.len(), 1);
        assert_eq!(workflows[0].models[0].id, "LIMITED-MODEL");
    }

    #[test]
    fn it_preserves_job_order_and_propagates_errors() {
        let jobs: Vec<_> = (0..20)
            .map(|i| move || async move { Ok::<_, ()>(i * 2) })
            .collect();
        let results = futures::executor::block_on(run_limited(jobs)).unwrap();
        let expected: Vec<_> = (0..20).map(|i| i * 2).collect();
        assert_eq!(results, expected);

        let failing: Vec<_> = (0..3)
            .map(|i| move || async move { if i == 1 { Err("boom") } else { Ok(i) } })
            .collect();
        let error = futures::executor::block_on(run_limited(failing)).unwrap_err();
        assert_eq!(error, "boom");
    }

    #[test]
    fn it_formats_join_errors_with_or_without_response_body() {
        use geoengine_api_client::apis::{Error as ApiError, ResponseContent};

        // `reqwest::StatusCode` is a re-export of the shared `http` type, which axum
        // also re-exports — no extra dev-dependency needed to build the error.
        let with_response: ApiError<()> = ApiError::ResponseError(ResponseContent {
            status: axum::http::StatusCode::BAD_REQUEST,
            content: r#"{"error":"bad","message":"nope"}"#.to_string(),
            entity: None,
        });
        let formatted = unpack_join_error(&with_response, "register a workflow").to_string();
        assert!(formatted.contains("register a workflow"), "{formatted}");
        assert!(formatted.contains("bad"), "{formatted}");

        let without_response: ApiError<()> =
            ApiError::Serde(serde_json::from_str::<serde_json::Value>("oops").unwrap_err());
        let formatted = unpack_join_error(&without_response, "execute a workflow").to_string();
        assert!(formatted.contains("execute a workflow"), "{formatted}");
    }

    fn workflow_request(variable: ClimateVariable, scenario: ClimateScenario) -> WorkflowRequest {
        WorkflowRequest {
            models: vec![ClimateModelProperties {
                id: "ACCESS-CM2".to_string(),
                variant: "r1i1p1f1".to_string(),
                grid: "gn".to_string(),
                scenarios: vec![
                    ClimateScenario::Historical,
                    ClimateScenario::Ssp245,
                    ClimateScenario::Ssp585,
                ],
            }],
            variable,
            scenario: scenario.properties(),
            // `ProcessingGraph::default()` recurses forever (the generated `VectorOperator::default`
            // is `ColumnRangeFilter`, whose `sources` nests another vector operator), so build the
            // leaf-only graph the default is *meant* to produce.
            workflow: geoengine_api_client::models::ProcessingGraph::TypedVectorOperator(Box::new(
                geoengine_api_client::models::TypedVectorOperator {
                    operator: Box::new(
                        geoengine_api_client::models::VectorOperator::MockPointSource(
                            Default::default(),
                        ),
                    ),
                    r#type: Default::default(),
                },
            )),
        }
    }

    fn single_feature_collection(properties: &serde_json::Value) -> GeoJson {
        GeoJson {
            features: vec![serde_json::json!({"type": "Feature", "properties": properties})],
            r#type: CollectionType::FeatureCollection,
        }
    }

    #[test]
    fn it_derives_anomalies_and_raw_members_from_reference_values() {
        let request = workflow_request(ClimateVariable::HeatDays, ClimateScenario::Ssp245);
        let analysis = WfsQueryResult {
            geo_json: single_feature_collection(&serde_json::json!({"ACCESS-CM2": 20.0})),
            computation_id: None,
        };
        let reference = WfsQueryResult {
            geo_json: single_feature_collection(&serde_json::json!({"ACCESS-CM2": 12.5})),
            computation_id: None,
        };

        let (rows, raw_rows) = aggregate_rows(
            std::slice::from_ref(&analysis),
            &[reference],
            &[request],
            &[workflow_request(
                ClimateVariable::HeatDays,
                ClimateScenario::Historical,
            )],
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_abs_diff_eq!(row.anomaly.unwrap().0, 7.5);
        // the single reference year is the level; the analysis year exceeds it
        assert_eq!(row.occurrence_probability, Some(1.0));
        assert_eq!(row.risk_ratio, None);
        assert!(row.anomaly_label.is_some());
        assert!(row.anomaly_color.is_some());
        assert_abs_diff_eq!(row.mean.0, 20.0);
        // A single model yields an odd member count; the median is the middle value.
        assert_abs_diff_eq!(row.median.0, 20.0);
        assert_eq!(raw_rows.len(), 1);
        assert_eq!(raw_rows[0].model, "ACCESS-CM2");
        assert_abs_diff_eq!(raw_rows[0].value, 20.0);
    }

    #[test]
    fn it_omits_the_anomaly_without_reference_values() {
        let request = workflow_request(ClimateVariable::HeatDays, ClimateScenario::Ssp245);
        let analysis = WfsQueryResult {
            geo_json: single_feature_collection(&serde_json::json!({"ACCESS-CM2": 20.0})),
            computation_id: None,
        };
        // An empty feature collection cannot yield reference values.
        let broken_reference = WfsQueryResult {
            geo_json: GeoJson::default(),
            computation_id: None,
        };

        let (rows, _) = aggregate_rows(
            std::slice::from_ref(&analysis),
            &[broken_reference],
            &[request],
            &[workflow_request(
                ClimateVariable::HeatDays,
                ClimateScenario::Historical,
            )],
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert!(rows[0].anomaly.is_none());
        assert!(rows[0].occurrence_probability.is_none());
        assert!(rows[0].anomaly_label.is_none());
        assert!(rows[0].anomaly_color.is_none());
    }

    #[test]
    fn it_fails_without_analysis_values() {
        let request = workflow_request(ClimateVariable::HeatDays, ClimateScenario::Ssp245);
        let empty = WfsQueryResult {
            geo_json: GeoJson::default(),
            computation_id: None,
        };

        let result = aggregate_rows(&[empty], &[], &[request], &[]);

        assert!(result.is_err());
    }

    #[test]
    fn it_logs_workflow_requests_and_results() {
        let request = workflow_request(ClimateVariable::HeatDays, ClimateScenario::Ssp245);

        let result = WfsQueryResult {
            geo_json: single_feature_collection(&serde_json::json!({"ACCESS-CM2": 1.0})),
            computation_id: None,
        };
        log_wfs_results(std::slice::from_ref(&request), &[result]);
    }

    #[test]
    fn it_skips_missing_or_invalid_model_columns() {
        let models = vec![
            ClimateModelProperties {
                id: "MPI-ESM1-2-LR".to_string(),
                variant: "r1i1p1f1".to_string(),
                grid: "gn".to_string(),
                scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
            },
            ClimateModelProperties {
                id: "ACCESS-CM2".to_string(),
                variant: "r1i1p1f1".to_string(),
                grid: "gn".to_string(),
                scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
            },
        ];
        let geo_json = GeoJson {
            features: vec![
                serde_json::json!({"type": "Feature", "properties": {"MPI-ESM1-2-LR": "not-a-number", "ACCESS-CM2": 4.0}}),
                serde_json::json!({"type": "Feature", "properties": {"MPI-ESM1-2-LR": 8.0}}),
            ],
            r#type: CollectionType::FeatureCollection,
        };

        let result = outputs_from_feature_collection(&geo_json, &models).unwrap();

        assert_abs_diff_eq!(result["MPI-ESM1-2-LR"].mean, 8.0);
        assert_abs_diff_eq!(result["ACCESS-CM2"].mean, 4.0);
    }

    /// Full live check against a running `GeoEngine`: registers the real workflow
    /// (yearly-count `MultiBandGdalSource` -> `RasterVectorJoin`) and queries heat/ice days over the
    /// 2015-2019 analysis and the 1950-1954 reference. Only runs on demand -
    /// `cargo test -- --ignored it_runs_a_*`.
    ///
    /// Requires `nexgddp_cmip6_ACCESS-CM2_{historical,ssp245}_{heatDays,iceDays}_yearly`, built by
    /// `notebooks/nexgddp_cmip6_aggregate_yearly.ipynb` in `DEV_ONLY` mode, and a reachable
    /// `GeoEngine` at `localhost:3030`:
    #[tokio::test]
    #[ignore = "requires a live GeoEngine with the NEX-GDDP-CMIP6 datasets registered"]
    async fn it_runs_a_climate_risk_query_against_live_geoengine() {
        let mut configuration = geoengine_api_client::apis::configuration::Configuration::new();
        configuration.base_path = "http://localhost:3030/api".into();

        // Workflow registration requires an authenticated session (anonymous read covers the
        // data, not the workflows).
        let session = geoengine_api_client::apis::session_api::login_handler(
            &configuration,
            geoengine_api_client::models::UserCredentials {
                email: "admin@localhost".to_string(),
                password: "adminadmin".to_string(),
            },
        )
        .await
        .expect("GeoEngine admin login failed");
        configuration.bearer_access_token = Some(session.id.to_string());

        let coordinate = PointType::from((13.4050, 52.5200)); // Berlin: a known land cell
        let periods = ClimatePeriods {
            analysis_start: Year(2015),
            range: YearRange(5),
            reference_start: Year(1950),
        };
        let models = vec![ClimateModelProperties {
            id: "ACCESS-CM2".to_string(),
            variant: "r1i1p1f1".to_string(),
            grid: "gn".to_string(),
            scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
        }];

        let (outputs, computation_ids) = compute_climate(
            &configuration,
            &coordinate,
            &periods,
            &[ClimateVariable::HeatDays, ClimateVariable::IceDays],
            &[ClimateScenario::Ssp245],
            &models,
        )
        .await
        .expect("climate-risk query against the live GeoEngine failed");

        let rows = &outputs.climate_risk;
        assert_eq!(rows.len(), 2, "expected one row per variable: {rows:?}");
        for row in rows {
            let mean = row.mean.0;
            assert!(
                (0.0..=366.0).contains(&mean),
                "daily count out of bounds: {row:?}"
            );
            assert!(
                row.anomaly.is_some(),
                "expected an anomaly against the reference period: {row:?}"
            );
        }
        assert!(
            !computation_ids.is_empty(),
            "expected WFS computation ids for credit accounting"
        );
    }
}
