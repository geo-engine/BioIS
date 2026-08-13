use crate::{
    config::CONFIG,
    processes::parameters::{DataResource, PointGeoJsonInput, Year, YearRange},
    state::USER,
};
use anyhow::{Context, Result};
use geojson::PointType;
use ogcapi::{
    processes::Processor,
    types::{
        common::Link,
        processes::{
            Execute, ExecuteResults, JobControlOptions, Process, ProcessSummary, TransmissionMode,
            description::{DescriptionType, InputDescription, Metadata, OutputDescription},
        },
    },
};
use schemars::generate::SchemaSettings;
use std::collections::HashMap;

mod compute;
mod types;
mod workflow;

use self::{compute::*, types::*};
/// Calculates climate-risk indicators for a given point and time window.
#[derive(Debug, Clone)]
pub struct ClimateRiskProcess;

#[cfg(test)]
mod tests;
/// Generate the JSON Schema for the `models` input and attach `enumNames` hints
/// that list the scenarios each model is available for.
fn models_schema_with_hints(generator: &mut schemars::SchemaGenerator) -> serde_json::Value {
    let mut schema = generator.root_schema_for::<Vec<CordexModel>>().to_value();

    if let Some(items) = schema.get_mut("items").and_then(|i| i.as_object_mut())
        && let Some(enum_values) = items.get("enum").and_then(|e| e.as_array())
    {
        let enum_names: Vec<String> = enum_values
            .iter()
            .filter_map(|v| v.as_str())
            .map(|model_value| {
                CordexModel::ALL
                    .iter()
                    .find(|model| model.name() == model_value)
                    .map_or_else(
                        || model_value.to_string(),
                        |model| {
                            let props = model.properties();
                            let scenarios = props
                                .scenarios
                                .iter()
                                .map(|s| s.properties().name)
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("{} ({})", props.name, scenarios)
                        },
                    )
            })
            .collect();
        items.insert(
            "enumNames".to_string(),
            serde_json::to_value(enum_names).unwrap_or_default(),
        );
    }

    schema
}

#[async_trait::async_trait]
impl Processor for ClimateRiskProcess {
    fn id(&self) -> &'static str {
        "climate-risk"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    #[allow(clippy::too_many_lines)]
    fn process(&self) -> Result<Process> {
        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;
        let mut generator = settings.into_generator();

        let mut reference_year_begin_schema =
            generator.root_schema_for::<Option<Year>>().to_value();
        reference_year_begin_schema["default"] = serde_json::json!(DATA_START_YEAR);

        let inputs = HashMap::from([
            (
                "coordinate".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Coordinate in WGS84".to_string()),
                        description: Some("This is a POINT input in WGS84 (EPSG:4326) format.".to_string()),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<PointGeoJsonInput>().to_value(),
                    ..Default::default()
                },
            ),
            (
                "yearBegin".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Start year".to_string()),
                        description: Some("The first year to include in the climate-risk aggregation.".to_string()),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<Year>().to_value(),
                    ..Default::default()
                },
            ),
            (
                "yearRange".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Range (years)".to_string()),
                        description: Some(
                            "Length of the climate-risk aggregation window in years (5-30).".to_string(),
                        ),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<YearRange>().to_value(),
                    ..Default::default()
                },
            ),
            (
                "referenceYearBegin".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Reference period start".to_string()),
                        description: Some(
                            "First year of the reference period used to compute anomalies. Uses the same range as the analysis window. Disable the input to turn off anomaly computation.".to_string(),
                        ),
                        metadata: vec![Metadata {
                            title: None,
                            role: Some("enabled-by-default".to_string()),
                            href: None,
                        }],
                        ..Default::default()
                    },
                    schema: reference_year_begin_schema,
                    min_occurs: Some(0),
                    ..Default::default()
                },
            ),
            (
                "variables".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Climate variables".to_string()),
                        description: Some(
                            "The climate indicators to derive from the source dataset. If empty, all available indicators are computed.".to_string(),
                        ),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<Vec<ClimateVariable>>().to_value(),
                    min_occurs: Some(0),
                    ..Default::default()
                },
            ),
            (
                "models".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Climate models".to_string()),
                        description: Some("The climate-model workflows to execute for each requested variable.".to_string()),
                        ..Default::default()
                    },
                    schema: models_schema_with_hints(&mut generator),
                    ..Default::default()
                },
            ),
            (
                "region".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Climate data region".to_string()),
                        description: Some("The climate-data region to use for the risk aggregation. If not specified, the region will be inferred from the input coordinate.".to_string()),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<Option<CordexRegion>>().to_value(),
                    min_occurs: Some(0),
                    ..Default::default()
                },
            ),
        ]);

        let mut outputs = HashMap::from([
            (
                "inputs".to_string(),
                OutputDescription {
                    description_type: DescriptionType {
                        title: Some("Input parameters".to_string()),
                        description: Some(
                            "The inputs used to compute the climate-risk summary.".to_string(),
                        ),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<ClimateRiskInputs>().to_value(),
                },
            ),
            (
                "rawEnsembleData".to_string(),
                OutputDescription {
                    description_type: DescriptionType {
                        title: Some("Raw ensemble data".to_string()),
                        description: Some(
                            "Per-model raw values for each variable × scenario combination."
                                .to_string(),
                        ),
                        metadata: vec![Metadata {
                            title: None,
                            role: Some("default-disabled".to_string()),
                            href: None,
                        }],
                        ..Default::default()
                    },
                    schema: generator
                        .root_schema_for::<DataResource<Vec<ClimateRiskRawRow>>>()
                        .to_value(),
                },
            ),
        ]);

        for scenario in ClimateScenario::ALL {
            let props = scenario.properties();
            outputs.insert(
                scenario.name().to_string(),
                OutputDescription {
                    description_type: DescriptionType {
                        title: Some(props.name.to_string()),
                        description: Some(format!(
                            "A table of climate-risk indicators for the {} scenario.",
                            props.name
                        )),
                        ..Default::default()
                    },
                    schema: generator
                        .root_schema_for::<DataResource<Vec<ClimateRiskRow>>>()
                        .to_value(),
                },
            );
        }

        Ok(Process {
            summary: ProcessSummary {
                id: self.id().into(),
                version: self.version().into(),
                description: DescriptionType {
                    title: Some("Climate risk indicators".to_string()),
                    description: Some(
                        "This process derives climate-risk indicators such as heat days from CORDEX/CMIP5 climate data for a point location and a time window. The workflow builds a daily threshold mask, aggregates it over the requested years and returns summary statistics for the selected climate variable. An anomaly relative to a reference period (same length) is reported as the difference of the multi-year means. If no models are specified, all models compatible with the region are used. If no region is specified, it is automatically inferred from the coordinate. If no scenario outputs are requested, all scenarios are computed."
                            .to_string(),
                    ),
                    ..Default::default()
                },
                job_control_options: vec![
                    JobControlOptions::SyncExecute,
                    JobControlOptions::AsyncExecute,
                ],
                output_transmission: vec![TransmissionMode::Value],
                links: vec![Link::new(
                    format!("./{}/execution", self.id()),
                    "http://www.opengis.net/def/rel/ogc/1.0/execute",
                )
                .title("Execution endpoint")],
            },
            inputs,
            outputs,
        })
    }

    async fn execute(&self, execute: Execute) -> Result<ExecuteResults> {
        let mut inputs = parse_inputs(&execute.inputs)?;

        validate_inputs(
            inputs.year_begin,
            inputs.year_range,
            inputs.reference_year_begin,
        )?;

        let point = inputs.coordinate.value.coordinates.clone();

        let region_props = resolve_region(inputs.region, &point)?;
        if inputs.region.is_none() {
            inputs.region = Some(region_props.region);
        }

        let (filtered_models, model_props, dropped_models) =
            resolve_models(&inputs.models, region_props.region);
        if !dropped_models.is_empty() {
            tracing::warn!(
                "Ignoring climate models not available for region {}: {}",
                region_props.name,
                dropped_models.join(", ")
            );
        }
        if model_props.is_empty() {
            let detail = if dropped_models.is_empty() {
                String::new()
            } else {
                format!(
                    "; none of the requested models ({}) are available",
                    dropped_models.join(", ")
                )
            };
            anyhow::bail!(
                "No climate models valid / available for the specified region: {}{detail}",
                region_props.name
            );
        }
        inputs.models = filtered_models;

        let scenario_props = resolve_available_scenarios(&model_props);
        if scenario_props.is_empty() {
            anyhow::bail!(
                "No climate scenarios valid / available for the specified region and models."
            );
        }

        let available_scenarios: Vec<ClimateScenario> =
            scenario_props.iter().map(|s| s.scenario).collect();

        let output_keys: std::collections::BTreeSet<String> =
            execute.outputs.keys().cloned().collect();
        let (selected_scenarios, should_reflect_inputs, include_raw_ensemble) =
            resolve_requests(&output_keys, &available_scenarios)?;

        let variables = resolve_variables(&inputs.variables);
        let requests: Vec<_> = selected_scenarios
            .into_iter()
            .flat_map(|scenario| {
                variables
                    .iter()
                    .map(move |v| (ClimateVariableRequest::new(*v), scenario))
            })
            .collect();

        let request_props: Vec<(ClimateVariableRequest, ClimateScenarioProperties)> = requests
            .into_iter()
            .map(|(v, s)| (v, s.properties()))
            .collect();

        let mut outputs = compute_climate(
            &CONFIG
                .geoengine
                .api_config(USER.try_get().ok().map(|user| user.session_token)),
            &point,
            inputs.year_begin,
            inputs.year_range,
            inputs.reference_year_begin,
            &request_props,
            &model_props,
            &region_props,
        )
        .await?;

        if should_reflect_inputs {
            outputs.inputs = Some(inputs);
        }
        if !include_raw_ensemble {
            outputs.raw_ensemble_data = None;
        }

        Ok(outputs.into())
    }
}

fn parse_inputs(
    inputs: &HashMap<String, ogcapi::types::processes::Input>,
) -> Result<ClimateRiskInputs> {
    let value = serde_json::to_value(inputs).context("Failed to serialize process inputs")?;
    serde_json::from_value(value).context("Failed to deserialize climate-risk inputs")
}

fn validate_inputs(
    Year(start_year): Year,
    YearRange(range): YearRange,
    reference_year: Option<Year>,
) -> Result<()> {
    if !(5..=30).contains(&range) {
        anyhow::bail!("Year range must be between 5 and 30 years");
    }
    if start_year < DATA_START_YEAR {
        anyhow::bail!("Start year must be at least {DATA_START_YEAR}");
    }
    if start_year + range > 2100 {
        anyhow::bail!("Start year plus range must not exceed 2100");
    }
    if let Some(Year(reference_year)) = reference_year {
        if reference_year < DATA_START_YEAR {
            anyhow::bail!("Reference period start year must be at least {DATA_START_YEAR}");
        }
        if reference_year + range > 2100 {
            anyhow::bail!("Reference period start year plus range must not exceed 2100");
        }
    }
    Ok(())
}

fn resolve_region(
    region: Option<CordexRegion>,
    point: &PointType,
) -> Result<CordexRegionProperties> {
    if let Some(r) = region {
        let props = r.properties();
        if !props.bounding_box.contains(point) {
            anyhow::bail!(
                "Coordinate is outside of the specified CORDEX/CMIP5 region: {}",
                props.name
            );
        }
        Ok(props)
    } else {
        let region = CordexRegion::point_to_region(point).ok_or_else(|| {
            anyhow::anyhow!("Coordinate is outside of the supported CORDEX/CMIP5 regions")
        })?;
        Ok(region.properties())
    }
}

/// Filters the requested models down to those of the given region. The third return value
/// lists the user-specified models that were dropped, so callers can report them.
fn resolve_models(
    specified_models: &[CordexModel],
    region: CordexRegion,
) -> (Vec<CordexModel>, Vec<CordexModelProperties>, Vec<String>) {
    if specified_models.is_empty() {
        let (models, props): (Vec<_>, Vec<_>) = CordexModel::ALL
            .iter()
            .map(|m| (*m, m.properties()))
            .filter(|(_, p)| p.region == region)
            .unzip();
        (models, props, Vec::new())
    } else {
        let mut models = Vec::new();
        let mut props = Vec::new();
        let mut dropped = Vec::new();
        for model in specified_models {
            let model_props = model.properties();
            if model_props.region == region {
                models.push(*model);
                props.push(model_props);
            } else {
                dropped.push(model_props.name.to_string());
            }
        }
        (models, props, dropped)
    }
}

fn resolve_available_scenarios(
    model_props: &[CordexModelProperties],
) -> Vec<ClimateScenarioProperties> {
    ClimateScenario::ALL
        .iter()
        .copied()
        .filter(|s| model_props.iter().any(|m| m.scenarios.contains(s)))
        .map(ClimateScenario::properties)
        .collect()
}

fn resolve_variables(specified_variables: &[ClimateVariable]) -> Vec<ClimateVariable> {
    if specified_variables.is_empty() {
        ClimateVariable::ALL.to_vec()
    } else {
        specified_variables.to_vec()
    }
}

fn resolve_requests(
    output_keys: &std::collections::BTreeSet<String>,
    available_scenarios: &[ClimateScenario],
) -> Result<(Vec<ClimateScenario>, bool, bool)> {
    let mut should_reflect_inputs = output_keys.is_empty();
    let mut include_raw_ensemble = false;
    let mut selected_scenarios = Vec::new();

    // BTreeSet iterates in order, so scenario selection is deterministic.
    for output_key in output_keys {
        if output_key == "inputs" {
            should_reflect_inputs = true;
            continue;
        }

        if output_key == "rawEnsembleData" {
            include_raw_ensemble = true;
            continue;
        }

        let mut found = false;
        for scenario in ClimateScenario::ALL.iter().copied() {
            if scenario.name() == output_key && available_scenarios.contains(&scenario) {
                selected_scenarios.push(scenario);
                found = true;
                break;
            }
        }
        if !found {
            anyhow::bail!("Unknown output requested: {output_key}");
        }
    }

    if selected_scenarios.is_empty() {
        selected_scenarios = available_scenarios.to_vec();
    }

    Ok((
        selected_scenarios,
        should_reflect_inputs,
        include_raw_ensemble,
    ))
}
