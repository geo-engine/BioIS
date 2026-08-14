use crate::profile::CLIMATE_RISK_TABLE_SCHEMA_PROFILE;
use crate::{
    processes::parameters::{
        BioisDisplayKind, BioisDisplayMetadata, BioisTableSchemaExtension, BoundingBox,
        DataResource, Fields, TableSchemaField, TableSchemaType, Year, YearRange,
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
        schema: Fields {
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
    BioisTableSchemaExtension {
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
            let mut rows_by_scenario: std::collections::BTreeMap<String, Vec<ClimateRiskRow>> =
                std::collections::BTreeMap::new();
            for row in climate_risk.data {
                rows_by_scenario
                    .entry(row.scenario.clone())
                    .or_default()
                    .push(row);
            }
            for (scenario, rows) in rows_by_scenario {
                let analysis_period = outputs.analysis_period.as_deref().unwrap_or("");
                match climate_risk_scenario_data_resource(
                    &scenario,
                    rows,
                    analysis_period,
                    outputs.reference_period.as_deref(),
                )
                .to_input_value()
                {
                    Ok(value) => {
                        result.insert(
                            scenario,
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
) -> Result<Vec<GeoJson>> {
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
    analysis_results: Vec<GeoJson>,
    reference_results: Option<&[GeoJson]>,
    workflow_requests: &[WorkflowRequest],
) -> Result<(Vec<ClimateRiskRow>, Vec<ClimateRiskRawRow>)> {
    let mut rows = Vec::new();
    let mut raw_rows = Vec::new();
    for (i, analysis) in analysis_results.into_iter().enumerate() {
        let request = &workflow_requests[i];
        let var_props = request.variable.properties();
        let model_values = outputs_from_feature_collection(&analysis, &request.models)?;
        if let Some(aggregated) = aggregate_from_list(&model_values) {
            let reference =
                reference_results.and_then(
                    |reference_results| match outputs_from_feature_collection(
                        &reference_results[i],
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
            let anomaly = reference.map(|reference_mean| aggregated.mean - reference_mean);
            let anomaly_pct =
                reference.map(|reference_mean| anomaly_pct(aggregated.mean, reference_mean));
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
                    .map(|(days, pct)| anomaly_label(days, pct)),
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

#[allow(clippy::too_many_arguments)]
#[instrument(skip(configuration), err(Debug))]
pub(crate) async fn compute_climate(
    configuration: &Configuration,
    coordinate: &PointType,
    Year(start_year): Year,
    YearRange(range): YearRange,
    reference_year: Option<Year>,
    requests: &[(ClimateVariableRequest, ClimateScenarioProperties)],
    models: &[CordexModelProperties],
    region: &CordexRegionProperties,
) -> Result<ClimateRiskOutputs> {
    const POINT_BBOX_HALF_SPAN: f64 = 0.0001;
    if requests.is_empty() || models.is_empty() {
        return Ok(ClimateRiskOutputs::default());
    }

    let end_analysis = start_year + range;
    let time_str_analysis =
        format!("{start_year:04}-01-01T00:00:00Z/{end_analysis:04}-01-01T00:00:00Z");
    let analysis_period = format!("{start_year:04}–{:04}", end_analysis - 1);
    let reference_time = reference_year.map(|Year(reference_year)| {
        let end_reference = reference_year + range;
        format!("{reference_year:04}-01-01T00:00:00Z/{end_reference:04}-01-01T00:00:00Z")
    });
    let reference_period = reference_year
        .map(|Year(reference_year)| format!("{reference_year:04}–{}", reference_year + range - 1));
    let bbox = BoundingBox::around_point(coordinate, POINT_BBOX_HALF_SPAN);
    let bbox_string = bbox.wfs_string();

    let workflow_requests = build_workflows(coordinate, requests, models, region);
    let workflow_ids = register_workflows(configuration, &workflow_requests).await?;

    for (workflow_id, request) in workflow_ids.iter().zip(&workflow_requests) {
        tracing::debug!(
            "ClimateRisk: registered workflow: variable={:?}, scenario={:?}, workflow_id={}",
            request.variable.name(),
            request.scenario.scenario.name(),
            workflow_id,
        );
    }

    let analysis_results = query_workflows(
        configuration,
        &workflow_ids,
        &bbox_string,
        &time_str_analysis,
    )
    .await?;

    let reference_results = match &reference_time {
        Some(reference_time) => {
            Some(query_workflows(configuration, &workflow_ids, &bbox_string, reference_time).await?)
        }
        None => None,
    };

    for (i, analysis) in analysis_results.iter().enumerate() {
        for (j, feature) in analysis.features.iter().enumerate() {
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

    let (rows, raw_rows) = aggregate_rows(
        analysis_results,
        reference_results.as_deref(),
        &workflow_requests,
    )?;

    let climate_risk = Some(climate_risk_data_resource(
        rows,
        &analysis_period,
        reference_period.as_deref(),
    ));

    Ok(ClimateRiskOutputs {
        analysis_period: Some(analysis_period),
        reference_period,
        climate_risk,
        raw_ensemble_data: if raw_rows.is_empty() {
            None
        } else {
            Some(raw_ensemble_data_resource(raw_rows))
        },
        inputs: None,
    })
}

async fn wfs_query(
    configuration: &Configuration,
    workflow_id: &str,
    bbox: &str,
    time: &str,
) -> Result<GeoJson, geoengine_api_client::apis::Error<WfsHandlerError>> {
    wfs_handler(
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
    )
    .await
}

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
        max,
        min,
        mean,
        median,
        occurrence_probability: Some(mean / DAYS_PER_JULIAN_YEAR),
        raw_members: Some(raw_members),
    })
}

const NO_MODEL_COVERAGE_ERROR: &str =
    "Input coordinate not covered by any of the requested climate models for the given time range.";

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
