use crate::db::model::ComputationId;
use crate::profile::CLIMATE_RISK_TABLE_SCHEMA_PROFILE;
use crate::{
    processes::parameters::{
        BioISTableSchemaExtension, BioisDisplayKind, BioisDisplayMetadata, BoundingBox,
        DataResource, Days, TableSchema, TableSchemaField, TableSchemaType, Year, YearRange,
    },
    util::{error_response, to_api_vector_process},
};
use anyhow::Result;
use futures::{TryStreamExt, stream::StreamExt};
use geoengine_api_client::{
    apis::{
        configuration::Configuration, ogcwfs_api::WfsHandlerError, ogcwfs_api::wfs_handler,
        workflows_api::register_workflow_handler,
    },
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
use std::collections::HashMap;
use std::str::FromStr;
use tracing::instrument;

use super::ClimateRiskProcess;
use super::types::*;
pub(crate) fn climate_risk_data_resource(
    rows: Vec<ClimateRiskRow>,
    analysis_period: &str,
    reference_period: Option<&str>,
) -> DataResource<Vec<ClimateRiskRow>> {
    let mut fields = vec![TableSchemaField {
        name: "scenario".into(),
        r#type: Some(TableSchemaType::String),
        title: Some("Scenario".into()),
        ..Default::default()
    }];
    fields.extend(risk_fields(&rows, reference_period));
    let name = if analysis_period.is_empty() {
        "Climate Risk".to_string()
    } else {
        format!("Climate Risk · {analysis_period}")
    };
    let biois = climate_display_extension(&rows);
    DataResource {
        name,
        data: rows,
        schema: TableSchema {
            fields,
            primary_key: Some(vec!["variable".to_string(), "scenario".to_string()]),
            schema: Some(CLIMATE_RISK_TABLE_SCHEMA_PROFILE.to_string()),
            biois: Some(biois),
        },
    }
}

pub(crate) fn climate_risk_scenario_data_resource(
    scenario_name: &str,
    rows: Vec<ClimateRiskRow>,
    analysis_period: &str,
    reference_period: Option<&str>,
) -> DataResource<Vec<ClimateRiskRow>> {
    let fields = risk_fields(&rows, reference_period);
    let name = if analysis_period.is_empty() {
        scenario_name.to_string()
    } else {
        format!("{scenario_name} · {analysis_period}")
    };
    let biois = climate_display_extension(&rows);
    DataResource {
        name,
        data: rows,
        schema: TableSchema {
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
            title: Some("Occurrence Probability".into()),
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
        name: "Raw Ensemble Data".to_string(),
        data: rows,
        schema: TableSchema {
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

fn climate_display_extension(rows: &[ClimateRiskRow]) -> BioISTableSchemaExtension {
    let has_anomaly = rows.iter().any(|row| row.anomaly.is_some());
    let mut display = HashMap::from([(
        "occurrenceProbability".to_string(),
        BioisDisplayMetadata {
            kind: BioisDisplayKind::RiskProbability,
            label_field: Some("occurrenceProbabilityLabel".to_string()),
            color_field: Some("occurrenceProbabilityColor".to_string()),
        },
    )]);
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
    BioISTableSchemaExtension {
        display,
        hidden_fields: [
            "occurrenceProbabilityLabel",
            "occurrenceProbabilityColor",
            "anomalyLabel",
            "anomalyColor",
        ]
        .into_iter()
        // The probability label/color columns are always hidden; the anomaly ones only
        // when an anomaly column is actually present.
        .filter(|field| {
            matches!(
                *field,
                "occurrenceProbabilityLabel" | "occurrenceProbabilityColor"
            ) || (has_anomaly && matches!(*field, "anomalyLabel" | "anomalyColor"))
        })
        .map(str::to_string)
        .collect(),
    }
}

impl From<ClimateRiskOutputs> for ExecuteResults {
    fn from(outputs: ClimateRiskOutputs) -> Self {
        let mut result = ExecuteResults::default();

        if let Some(inputs) = outputs.inputs
            && let Some(value) = build_inputs_value(&inputs)
        {
            result.insert("inputs".to_string(), value);
        }

        if let Some(climate_risk) = outputs.climate_risk {
            let analysis_period = outputs.analysis_period.as_deref().unwrap_or("");
            let reference_period = outputs.reference_period.as_deref();
            let mut rows_by_scenario: std::collections::BTreeMap<String, Vec<ClimateRiskRow>> =
                std::collections::BTreeMap::new();
            for row in climate_risk.data {
                rows_by_scenario
                    .entry(row.scenario.clone())
                    .or_default()
                    .push(row);
            }
            for (scenario, rows) in rows_by_scenario {
                match climate_risk_scenario_data_resource(
                    &scenario,
                    rows,
                    analysis_period,
                    reference_period,
                )
                .to_input_value()
                {
                    Ok(value) => {
                        result.insert(
                            scenario_output_id(&scenario),
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
                        "Failed to serialize the climate-risk output for scenario `{scenario}`: {error}"
                    ),
                }
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

        result
    }
}

/// Maps a scenario's display name (e.g. `"RCP 4.5 (Intermediate emissions)"`) to the output id
/// declared in the process description (`"rcp45"`), so that the keys of the execute response
/// match the declared output ids. Falls back to the given name when it is not a known
/// display name, which keeps rows carrying output ids directly working as well.
fn scenario_output_id(scenario: &str) -> String {
    ClimateScenario::ALL
        .iter()
        .find(|s| s.properties().name == scenario)
        .map_or_else(|| scenario.to_string(), |s| s.name().to_string())
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
    models: Vec<CordexModelProperties>,
    variable: ClimateVariable,
    scenario: ClimateScenarioProperties,
    workflow: geoengine_api_client::models::Workflow,
}

/// Builds one workflow per (variable, scenario) pair, using only models that support the scenario.
fn build_workflows(
    coordinate: &PointType,
    requests: &[(ClimateVariableRequest, ClimateScenarioProperties)],
    models: &[CordexModelProperties],
    region: &CordexRegionProperties,
) -> Vec<WorkflowRequest> {
    requests
        .iter()
        .filter_map(|(var_req, scenario_props)| {
            let compatible_models: Vec<CordexModelProperties> = models
                .iter()
                .filter(|model| model.scenarios.contains(&scenario_props.scenario))
                .cloned()
                .collect();
            if compatible_models.is_empty() {
                return None;
            }

            let variable_properties = var_req.variable.properties();
            let raster_sources = compatible_models
                .iter()
                .map(|model| {
                    ClimateRiskProcess::build_variable_year_agg_workflow(
                        &variable_properties,
                        model,
                        scenario_props,
                        region,
                    )
                })
                .collect::<Vec<_>>();
            let model_var_names: Vec<String> = compatible_models
                .iter()
                .map(|model| model.model.name().to_string())
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
                variable: var_req.variable,
                scenario: scenario_props.clone(),
                workflow,
            })
        })
        .collect()
}

// bounds geoengine fan-out; the request count is finite but a single user
// request can register ~18 workflows and run ~36 WFS queries.
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

async fn register_workflows(
    configuration: &Configuration,
    requests: &[WorkflowRequest],
) -> Result<Vec<String>> {
    let workflow_ids = run_limited(
        requests
            .iter()
            .map(|request| {
                let workflow = &request.workflow;
                move || async move {
                    register_workflow_handler(configuration, workflow.clone())
                        .await
                        .map(|id| id.id.to_string())
                }
            })
            .collect(),
    )
    .await
    .map_err(|error| unpack_join_error(&error, "register a workflow"))?;
    Ok(workflow_ids)
}

async fn query_workflows(
    configuration: &Configuration,
    workflow_ids: &[String],
    bbox_string: &str,
    time: &str,
) -> Result<Vec<WfsQueryResult>> {
    run_limited(
        workflow_ids
            .iter()
            .map(|id| move || async move { wfs_query(configuration, id, bbox_string, time).await })
            .collect(),
    )
    .await
    .map_err(|error| unpack_join_error(&error, "execute a workflow"))
}

/// Aggregates the per-workflow WFS results into climate-risk and raw-ensemble rows.
fn aggregate_rows(
    analysis_results: &[WfsQueryResult],
    reference_results: Option<&[WfsQueryResult]>,
    workflow_requests: &[WorkflowRequest],
) -> Result<(Vec<ClimateRiskRow>, Vec<ClimateRiskRawRow>)> {
    let mut rows = Vec::new();
    let mut raw_rows = Vec::new();
    for (i, analysis) in analysis_results.iter().enumerate() {
        let request = &workflow_requests[i];
        let var_props = request.variable.properties();
        let model_values = outputs_from_feature_collection(&analysis.geo_json, &request.models)?;
        if let Some(aggregated) = aggregate_from_list(&model_values) {
            let reference =
                reference_results.and_then(
                    |reference_results| match outputs_from_feature_collection(
                        &reference_results[i].geo_json,
                        &request.models,
                    ) {
                        Ok(reference_values) => {
                            aggregate_from_list(&reference_values).map(|r| r.mean)
                        }
                        Err(error) => {
                            tracing::warn!(
                                "Failed to compute reference-period values for {}: {error}",
                                var_props.name_string()
                            );
                            None
                        }
                    },
                );
            let anomaly =
                reference.map(|reference_mean| Days(aggregated.mean.0 - reference_mean.0));
            let anomaly_pct =
                reference.map(|reference_mean| anomaly_pct(aggregated.mean.0, reference_mean.0));
            rows.push(ClimateRiskRow {
                variable: var_props.name_string(),
                scenario: request.scenario.name.to_string(),
                mean: aggregated.mean,
                median: aggregated.median,
                min: aggregated.min,
                max: aggregated.max,
                occurrence_probability: aggregated.occurrence_probability,
                anomaly,
                occurrence_probability_label: aggregated
                    .occurrence_probability
                    .map(probability_label),
                occurrence_probability_color: aggregated
                    .occurrence_probability
                    .map(probability_color),
                anomaly_label: anomaly
                    .zip(anomaly_pct)
                    .map(|(days, pct)| anomaly_label(days.0, pct)),
                anomaly_color: anomaly_pct.map(percentage_color),
            });
            if let Some(raw_members) = aggregated.raw_members {
                for (model_name, value) in raw_members {
                    raw_rows.push(ClimateRiskRawRow {
                        variable: var_props.name_string(),
                        scenario: request.scenario.name.to_string(),
                        model: model_name,
                        value,
                    });
                }
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

fn log_registered_workflows(workflow_ids: &[String], workflow_requests: &[WorkflowRequest]) {
    for (workflow_id, request) in workflow_ids.iter().zip(workflow_requests) {
        tracing::debug!(
            "ClimateRisk: registered workflow: variable={:?}, scenario={:?}, workflow_id={}",
            request.variable.name(),
            request.scenario.scenario.name(),
            workflow_id,
        );
    }
}

fn log_wfs_results(workflow_requests: &[WorkflowRequest], results: &[WfsQueryResult]) {
    for (i, result) in results.iter().enumerate() {
        for (j, feature) in result.geo_json.features.iter().enumerate() {
            if let Some(props) = feature.get("properties") {
                let request = &workflow_requests[i];
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
/// combinations, then aggregates the results into [`ClimateRiskOutputs`].
/// Also returns the geoengine computation ids of all executed WFS queries for credit accounting.
#[instrument(skip(configuration), err(Debug))]
pub(crate) async fn compute_climate(
    configuration: &Configuration,
    coordinate: &PointType,
    periods: &ClimatePeriods,
    requests: &[(ClimateVariableRequest, ClimateScenarioProperties)],
    models: &[CordexModelProperties],
    region: &CordexRegionProperties,
) -> Result<(ClimateRiskOutputs, Vec<ComputationId>)> {
    let Year(start_year) = periods.analysis_start;
    let YearRange(range) = periods.range;
    let Year(reference_year) = periods.reference_start;
    // ~11 m half-span: small enough to hit a single grid cell, large enough to avoid
    // floating-point edge cases on cell boundaries.
    const POINT_BBOX_HALF_SPAN: f64 = 0.0001;
    if requests.is_empty() || models.is_empty() {
        return Ok((ClimateRiskOutputs::default(), Vec::new()));
    }

    let end_analysis = start_year + range;
    let time_str_analysis =
        format!("{start_year:04}-01-01T00:00:00Z/{end_analysis:04}-01-01T00:00:00Z");
    let analysis_period = format!("{start_year:04}–{:04}", end_analysis - 1);
    let end_reference = reference_year + range;
    let reference_time =
        format!("{reference_year:04}-01-01T00:00:00Z/{end_reference:04}-01-01T00:00:00Z");
    let reference_period = format!("{reference_year:04}–{}", reference_year + range - 1);
    let bbox = BoundingBox::around_point(coordinate, POINT_BBOX_HALF_SPAN);
    let bbox_string = bbox.wfs_string();

    let workflow_requests = build_workflows(coordinate, requests, models, region);
    let workflow_ids = register_workflows(configuration, &workflow_requests).await?;
    log_registered_workflows(&workflow_ids, &workflow_requests);

    let analysis_results = query_workflows(
        configuration,
        &workflow_ids,
        &bbox_string,
        &time_str_analysis,
    )
    .await?;

    let reference_results =
        Some(query_workflows(configuration, &workflow_ids, &bbox_string, &reference_time).await?);
    log_wfs_results(&workflow_requests, &analysis_results);

    // Every WFS query is a geoengine computation that must be reported for credit accounting.
    let computation_ids: Vec<ComputationId> = analysis_results
        .iter()
        .chain(reference_results.iter().flatten())
        .filter_map(|result| result.computation_id)
        .collect();

    let (rows, raw_rows) = aggregate_rows(
        &analysis_results,
        reference_results.as_deref(),
        &workflow_requests,
    )?;

    let climate_risk = Some(climate_risk_data_resource(
        rows,
        &analysis_period,
        Some(reference_period.as_str()),
    ));
    Ok((
        ClimateRiskOutputs {
            analysis_period: Some(analysis_period),
            reference_period: Some(reference_period),
            climate_risk,
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
    )
}

/// Computes min, max, mean, and median across model values for a single variable/scenario.
#[allow(
    clippy::cast_precision_loss,
    reason = "Averaging over at most a few dozen model values cannot lose relevant precision."
)]
pub(crate) fn aggregate_from_list(
    model_values: &HashMap<CordexModel, f64>,
) -> Option<ClimateVariableResult> {
    if model_values.is_empty() {
        return None;
    }

    let values: Vec<f64> = model_values.values().copied().collect();
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

    let raw_members = model_values
        .iter()
        .map(|(k, v)| (k.name().to_string(), *v))
        .collect();

    Some(ClimateVariableResult {
        max: Days(max),
        min: Days(min),
        mean: Days(mean),
        median: Days(median),
        occurrence_probability: Some(mean / DAYS_PER_JULIAN_YEAR),
        raw_members: Some(raw_members),
    })
}

const NO_MODEL_COVERAGE_ERROR: &str =
    "Input coordinate not covered by any of the requested climate models for the given time range.";

/// Extracts per-model values from a WFS feature collection response.
#[allow(
    clippy::cast_precision_loss,
    reason = "Climate indicator values are small; i64→f64 and count→f64 cannot lose relevant precision."
)]
pub(crate) fn outputs_from_feature_collection(
    feature_collection: &GeoJson,
    variables: &[CordexModelProperties],
) -> Result<HashMap<CordexModel, f64>> {
    if feature_collection.features.is_empty() {
        anyhow::bail!(NO_MODEL_COVERAGE_ERROR);
    }

    // One feature per time step (year); average each model's per-year values to get the
    // multi-year mean.
    let mut acc: HashMap<CordexModel, Vec<f64>> = HashMap::new();
    let mut models_without_data = Vec::new();
    let mut models_with_invalid_data = Vec::new();

    for feature in &feature_collection.features {
        let Some(properties) = feature.get("properties") else {
            continue;
        };

        for model in variables {
            let model_name = model.model.name();
            if let Some(value) = properties.get(model_name) {
                if let Some(value) = value.as_f64().or_else(|| value.as_i64().map(|v| v as f64)) {
                    acc.entry(model.model).or_default().push(value);
                } else if !models_with_invalid_data.contains(&model_name) {
                    models_with_invalid_data.push(model_name);
                }
            } else if !models_without_data.contains(&model_name) {
                models_without_data.push(model_name);
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
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            (model, mean)
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
        fn row(scenario: &str) -> ClimateRiskRow {
            ClimateRiskRow {
                variable: "Heat Days".to_string(),
                scenario: scenario.to_string(),
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
            reference_period: Some("2006–2025".to_string()),
            climate_risk: Some(climate_risk_data_resource(
                vec![
                    row("RCP 2.6 (Low emissions)"),
                    row("RCP 4.5 (Intermediate emissions)"),
                    row("RCP 2.6 (Low emissions)"),
                ],
                "2041–2070",
                Some("2006–2025"),
            )),
            raw_ensemble_data: None,
        };

        let result: ExecuteResults = outputs.into();
        // Keys must be the output ids declared in the process description, not the
        // display names the rows carry.
        assert!(result.contains_key("rcp26"));
        assert!(result.contains_key("rcp45"));
        assert!(!result.contains_key("rcp85"));
        assert!(!result.contains_key("RCP 2.6 (Low emissions)"));

        let InlineOrRefData::QualifiedInputValue(qualified) = &result["rcp26"].data else {
            panic!("expected qualified input value");
        };
        let resource: DataResource<Vec<ClimateRiskRow>> =
            serde_json::from_value(serde_json::to_value(&qualified.value).unwrap()).unwrap();
        assert_eq!(resource.data.len(), 2);
        assert_eq!(resource.name, "RCP 2.6 (Low emissions) · 2041–2070");
        assert!(
            resource
                .data
                .iter()
                .all(|r| r.scenario == "RCP 2.6 (Low emissions)")
        );
    }

    #[test]
    fn it_scenario_output_id_maps_display_name_to_declared_output_id() {
        assert_eq!(
            scenario_output_id("RCP 4.5 (Intermediate emissions)"),
            "rcp45"
        );
        assert_eq!(scenario_output_id("RCP 8.5 (High emissions)"), "rcp85");
        // Unknown names pass through, so rows already carrying output ids keep working.
        assert_eq!(scenario_output_id("rcp45"), "rcp45");
    }

    #[test]
    fn it_aggregate_from_list_aggregates_values() {
        let mut values: HashMap<CordexModel, f64> = HashMap::new();
        values.insert(CordexModel::MpiMmpiEsmLr, 10.0);
        values.insert(CordexModel::MohcHadgem2Es, 20.0);

        let result = aggregate_from_list(&values).unwrap();
        assert_abs_diff_eq!(result.min.0, 10.0);
        assert_abs_diff_eq!(result.max.0, 20.0);
        assert_abs_diff_eq!(result.mean.0, 15.0);
        assert_abs_diff_eq!(result.median.0, 15.0);
        assert_eq!(result.raw_members.as_ref().unwrap().len(), 2);
        assert_abs_diff_eq!(result.occurrence_probability.unwrap(), 15.0 / 365.25);

        let empty: HashMap<CordexModel, f64> = HashMap::new();
        assert!(aggregate_from_list(&empty).is_none());
    }

    #[test]
    fn it_climate_risk_data_resource_declares_display_extension() {
        let rows = vec![ClimateRiskRow {
            variable: "Heat Days".to_string(),
            scenario: "rcp45".to_string(),
            max: Days(100.0),
            min: Days(0.0),
            mean: Days(50.0),
            median: Days(50.0),
            occurrence_probability: Some(0.5),
            anomaly: Some(Days(10.0)),
            ..Default::default()
        }];
        let resource = climate_risk_data_resource(rows, "", None);
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
                scenario: "rcp45".to_string(),
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
                scenario: "rcp45".to_string(),
                max: Days(100.0),
                min: Days(0.0),
                mean: Days(50.0),
                median: Days(50.0),
                occurrence_probability: Some(0.5),
                anomaly: Some(Days(5.0)),
                ..Default::default()
            },
        ];
        let resource = climate_risk_data_resource(rows, "", None);

        let field_names: std::collections::HashSet<&str> = resource
            .schema
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        let extension = resource.schema.biois.as_ref().unwrap();
        for metadata in extension.display.values() {
            assert!(field_names.contains(metadata.label_field.as_deref().unwrap()));
            assert!(field_names.contains(metadata.color_field.as_deref().unwrap()));
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
            scenario: "rcp45".to_string(),
            max: Days(100.0),
            min: Days(0.0),
            mean: Days(50.0),
            median: Days(50.0),
            occurrence_probability: Some(0.5),
            anomaly: None,
            ..Default::default()
        }];
        let resource = climate_risk_data_resource(rows, "", None);
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
            scenario: "RCP 2.6 (Low emissions)".to_string(),
            max: Days(100.0),
            min: Days(0.0),
            mean: Days(50.0),
            median: Days(50.0),
            occurrence_probability: Some(0.5),
            anomaly: Some(Days(10.0)),
            ..Default::default()
        }];
        let resource = climate_risk_scenario_data_resource(
            "RCP 2.6 (Low emissions)",
            rows,
            "2041–2070",
            Some("2006–2025"),
        );

        assert_eq!(resource.name, "RCP 2.6 (Low emissions) · 2041–2070");
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
        assert_eq!(
            title("anomaly").as_deref(),
            Some("Anomaly (days/year compared to 2006–2025)")
        );

        let resource = climate_risk_scenario_data_resource(
            "RCP 2.6 (Low emissions)",
            vec![ClimateRiskRow {
                variable: "Heat Days".to_string(),
                scenario: "RCP 2.6 (Low emissions)".to_string(),
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
        );
        assert_eq!(resource.name, "RCP 2.6 (Low emissions)");
        assert!(resource.schema.fields.iter().all(|f| f.name != "anomaly"));
    }

    #[test]
    fn it_converts_outputs_into_execute_results() {
        let rows = vec![ClimateRiskRow {
            variable: "Heat Days".to_string(),
            scenario: "rcp45".to_string(),
            max: Days(100.0),
            min: Days(0.0),
            mean: Days(50.0),
            median: Days(50.0),
            occurrence_probability: None,
            anomaly: Some(Days(10.0)),
            ..Default::default()
        }];
        let raw_rows = vec![ClimateRiskRawRow {
            variable: "Heat Days".to_string(),
            scenario: "rcp45".to_string(),
            model: "MPI-M-MPI-ESM-LR".to_string(),
            value: 42.0,
        }];
        let outputs = ClimateRiskOutputs {
            inputs: None,
            analysis_period: None,
            reference_period: None,
            climate_risk: Some(climate_risk_data_resource(rows, "", None)),
            raw_ensemble_data: Some(raw_ensemble_data_resource(raw_rows)),
        };
        let results: ExecuteResults = outputs.into();
        assert!(results.contains_key("rcp45"));
        assert!(!results.contains_key("rcp26"));
        assert!(!results.contains_key("rcp85"));
        assert!(results.contains_key("rawEnsembleData"));
    }

    #[test]
    fn it_averages_per_model_values_from_feature_collection() {
        let models = vec![
            CordexModel::MpiMmpiEsmLr.properties(),
            CordexModel::MohcHadgem2Es.properties(),
        ];

        let geo_json = GeoJson {
            features: vec![
                serde_json::json!({"type": "Feature", "properties": {"MPI-M-MPI-ESM-LR": 42.0, "MOHC-HadGEM2-ES": 10.0}}),
                serde_json::json!({"type": "Feature", "properties": {"MPI-M-MPI-ESM-LR": 58.0, "MOHC-HadGEM2-ES": 30.0}}),
            ],
            r#type: CollectionType::FeatureCollection,
        };
        let result = outputs_from_feature_collection(&geo_json, &models).unwrap();
        assert_eq!(result.len(), 2);
        assert_abs_diff_eq!(result[&CordexModel::MpiMmpiEsmLr], 50.0);
        assert_abs_diff_eq!(result[&CordexModel::MohcHadgem2Es], 20.0);

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
                scenario: "rcp45".to_string(),
                model: "MOHC-HadGEM2-ES".to_string(),
                value: 1.0,
            },
            ClimateRiskRawRow {
                variable: "Heat Days".to_string(),
                scenario: "rcp85".to_string(),
                model: "MPI-M-MPI-ESM-LR".to_string(),
                value: 2.0,
            },
            ClimateRiskRawRow {
                variable: "Heat Days".to_string(),
                scenario: "rcp26".to_string(),
                model: "MOHC-HadGEM2-ES".to_string(),
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
                ("Heat Days", "rcp26", "MOHC-HadGEM2-ES"),
                ("Heat Days", "rcp85", "MPI-M-MPI-ESM-LR"),
                ("Ice Days", "rcp45", "MOHC-HadGEM2-ES"),
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
            climate_risk: None,
            raw_ensemble_data: None,
        };

        let results: ExecuteResults = outputs.into();

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
        use ogcapi::types::common::Crs;

        let point = PointType::from(vec![8.0, 50.0]);
        let region = CordexRegionProperties {
            name: "Europe",
            dataset_prefix: "europe",
            bounding_box: BoundingBox::new(-10.0, 34.0, 30.0, 72.0, Crs::from_epsg(4326)),
            region: CordexRegion::Eur,
        };
        // Only supports RCP 2.6, so RCP 4.5 requests must be filtered out.
        let models = vec![CordexModelProperties {
            name: "LIMITED-MODEL",
            dataset_prefix: "limited-model",
            region: CordexRegion::Eur,
            scenarios: vec![ClimateScenario::Rcp26],
            model: CordexModel::MpiMmpiEsmLr,
        }];
        let requests = vec![
            (
                ClimateVariableRequest::new(ClimateVariable::HeatDays),
                ClimateScenario::Rcp26.properties(),
            ),
            (
                ClimateVariableRequest::new(ClimateVariable::HeatDays),
                ClimateScenario::Rcp45.properties(),
            ),
            (
                ClimateVariableRequest::new(ClimateVariable::IceDays),
                ClimateScenario::Rcp26.properties(),
            ),
        ];

        let workflows = build_workflows(&point, &requests, &models, &region);

        assert_eq!(workflows.len(), 2);
        assert!(
            workflows
                .iter()
                .all(|w| w.scenario.scenario == ClimateScenario::Rcp26)
        );
        assert_eq!(workflows[0].variable, ClimateVariable::HeatDays);
        assert_eq!(workflows[1].variable, ClimateVariable::IceDays);
        assert_eq!(workflows[0].models.len(), 1);
        assert_eq!(workflows[0].models[0].name, "LIMITED-MODEL");
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
            models: vec![CordexModel::MpiMmpiEsmLr.properties()],
            variable,
            scenario: scenario.properties(),
            workflow: geoengine_api_client::models::Workflow::default(),
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
        let request = workflow_request(ClimateVariable::HeatDays, ClimateScenario::Rcp45);
        let analysis = WfsQueryResult {
            geo_json: single_feature_collection(&serde_json::json!({"MPI-M-MPI-ESM-LR": 20.0})),
            computation_id: None,
        };
        let reference = WfsQueryResult {
            geo_json: single_feature_collection(&serde_json::json!({"MPI-M-MPI-ESM-LR": 12.5})),
            computation_id: None,
        };

        let (rows, raw_rows) = aggregate_rows(
            std::slice::from_ref(&analysis),
            Some(&[reference]),
            &[request],
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_abs_diff_eq!(row.anomaly.unwrap().0, 7.5);
        assert!(row.anomaly_label.is_some());
        assert!(row.anomaly_color.is_some());
        assert_abs_diff_eq!(row.mean.0, 20.0);
        // A single model yields an odd member count; the median is the middle value.
        assert_abs_diff_eq!(row.median.0, 20.0);
        assert_eq!(raw_rows.len(), 1);
        assert_eq!(raw_rows[0].model, "MPI-M-MPI-ESM-LR");
        assert_abs_diff_eq!(raw_rows[0].value, 20.0);
    }

    #[test]
    fn it_omits_the_anomaly_without_reference_values() {
        let request = workflow_request(ClimateVariable::HeatDays, ClimateScenario::Rcp45);
        let analysis = WfsQueryResult {
            geo_json: single_feature_collection(&serde_json::json!({"MPI-M-MPI-ESM-LR": 20.0})),
            computation_id: None,
        };
        // An empty feature collection cannot yield reference values.
        let broken_reference = WfsQueryResult {
            geo_json: GeoJson::default(),
            computation_id: None,
        };

        let (rows, _) = aggregate_rows(
            std::slice::from_ref(&analysis),
            Some(&[broken_reference]),
            &[request],
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert!(rows[0].anomaly.is_none());
        assert!(rows[0].anomaly_label.is_none());
        assert!(rows[0].anomaly_color.is_none());
    }

    #[test]
    fn it_fails_without_analysis_values() {
        let request = workflow_request(ClimateVariable::HeatDays, ClimateScenario::Rcp45);
        let empty = WfsQueryResult {
            geo_json: GeoJson::default(),
            computation_id: None,
        };

        let result = aggregate_rows(&[empty], None, &[request]);

        assert!(result.is_err());
    }

    #[test]
    fn it_logs_workflow_requests_and_results() {
        let request = workflow_request(ClimateVariable::HeatDays, ClimateScenario::Rcp45);
        log_registered_workflows(&["wf-id".to_string()], std::slice::from_ref(&request));

        let result = WfsQueryResult {
            geo_json: single_feature_collection(&serde_json::json!({"MPI-M-MPI-ESM-LR": 1.0})),
            computation_id: None,
        };
        log_wfs_results(std::slice::from_ref(&request), &[result]);
    }

    #[test]
    fn it_skips_missing_or_invalid_model_columns() {
        let models = vec![
            CordexModel::MpiMmpiEsmLr.properties(),
            CordexModel::MohcHadgem2Es.properties(),
        ];
        let geo_json = GeoJson {
            features: vec![
                serde_json::json!({"type": "Feature", "properties": {"MPI-M-MPI-ESM-LR": "not-a-number", "MOHC-HadGEM2-ES": 4.0}}),
                serde_json::json!({"type": "Feature", "properties": {"MPI-M-MPI-ESM-LR": 8.0}}),
            ],
            r#type: CollectionType::FeatureCollection,
        };

        let result = outputs_from_feature_collection(&geo_json, &models).unwrap();

        assert_abs_diff_eq!(result[&CordexModel::MpiMmpiEsmLr], 8.0);
        assert_abs_diff_eq!(result[&CordexModel::MohcHadgem2Es], 4.0);
    }
}
