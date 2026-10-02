use crate::{
    config::CONFIG,
    credits::add_credits_used_pending,
    db::DbHandle,
    processes::parameters::{DataResource, PointGeoJsonInput, Year, YearRange},
    state::{CONTEXT, TaskLocalContext},
};
use anyhow::{Context, Result};
use ogcapi::{
    processes::Processor,
    types::{
        common::Link,
        processes::{
            Execute, JobControlOptions, Process, ProcessSummary, TransmissionMode,
            description::{DescriptionType, InputDescription, OutputDescription},
        },
    },
};
use schemars::generate::SchemaSettings;
use std::collections::HashMap;
use std::sync::LazyLock;

mod compute;
mod types;
mod workflow;

pub use types::{ClimateRiskInputs, ClimateRiskOutputs, ClimateRiskRawRow, ClimateRiskRow};

use self::{compute::*, types::*};

// ponytail: model registry loaded once at startup from TOML; add models by editing conf/nexgddp_models.toml, no recompile
static MODEL_REGISTRY: LazyLock<HashMap<String, ClimateModelProperties>> =
    LazyLock::new(load_model_registry);

/// Reads the model registry, returning an empty one if it cannot be read or parsed.
///
/// A missing or broken registry must not take down the other processes, so this only warns; the
/// reason is kept in [`REGISTRY_PROBLEM`] so that a climate-risk execution can report *why* no
/// models are available instead of an unexplained empty selection.
fn load_model_registry() -> HashMap<String, ClimateModelProperties> {
    let path = &CONFIG.nexgddp_cmip6.model_registry_path;
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) => {
            let reason = format!("model registry `{path}` could not be read: {error}");
            tracing::warn!("{reason}, climate-risk will offer no models");
            REGISTRY_PROBLEM
                .lock()
                .expect("registry problem lock poisoned")
                .replace(reason);
            return HashMap::new();
        }
    };
    let registry: ModelRegistryToml = match toml::from_str(&content) {
        Ok(registry) => registry,
        Err(error) => {
            let reason = format!("model registry `{path}` could not be parsed: {error}");
            tracing::warn!("{reason}, climate-risk will offer no models");
            REGISTRY_PROBLEM
                .lock()
                .expect("registry problem lock poisoned")
                .replace(reason);
            return HashMap::new();
        }
    };
    let mut entries: Vec<(String, ClimateModelProperties)> = registry
        .model
        .into_iter()
        .map(|entry| {
            let scenarios: Vec<ClimateScenario> = entry
                .scenarios
                .into_iter()
                .filter_map(|s| match s.as_str() {
                    "historical" => Some(ClimateScenario::Historical),
                    "ssp245" => Some(ClimateScenario::Ssp245),
                    "ssp585" => Some(ClimateScenario::Ssp585),
                    _ => None,
                })
                .collect();
            (
                entry.id.clone(),
                ClimateModelProperties {
                    id: entry.id,
                    variant: entry.variant,
                    grid: entry.grid,
                    scenarios,
                },
            )
        })
        .collect();
    // ponytail: sort by model id for deterministic iteration order
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.into_iter().collect()
}

/// Why the registry is empty, if loading it failed, cf. [`load_model_registry`].
static REGISTRY_PROBLEM: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// The reason the registry could not be loaded, if it failed.
fn registry_problem() -> Option<String> {
    REGISTRY_PROBLEM
        .lock()
        .ok()
        .and_then(|problem| problem.clone())
}

#[derive(serde::Deserialize)]
struct ModelRegistryToml {
    model: Vec<ModelRegistryEntry>,
}

#[derive(serde::Deserialize)]
struct ModelRegistryEntry {
    id: String,
    variant: String,
    grid: String,
    scenarios: Vec<String>,
}

/// Calculates climate-risk indicators for a given point and time window.
#[derive(Debug, Clone)]
pub struct ClimateRiskProcess {
    db: DbHandle,
}

impl ClimateRiskProcess {
    pub const ID: &'static str = "climate-risk";

    #[must_use]
    pub fn new(db: DbHandle) -> Self {
        Self { db }
    }
}

/// Restricts the `models` input to the registered model ids.
///
/// The registry is loaded at runtime, so the enum cannot come from the `Vec<String>` type itself.
/// This is a schema transform rather than an inline mutation, cf. `optional_feature_properties`
/// and `relative_json_pointer_format`.
fn registered_models(schema: &mut schemars::Schema) {
    let Some(items) = schema
        .get_mut("items")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };
    items.insert(
        "enum".to_string(),
        serde_json::Value::Array(
            MODEL_REGISTRY
                .keys()
                .map(|id| serde_json::Value::String(id.clone()))
                .collect(),
        ),
    );
}

/// Generate the JSON Schema for the `models` input, enumerating the registered model ids so the
/// frontend can render a dropdown instead of a free-text list.
fn models_schema(generator: &mut schemars::SchemaGenerator) -> serde_json::Value {
    let mut root = generator.root_schema_for::<Vec<String>>();
    registered_models(&mut root);
    root.to_value()
}

fn build_inputs(generator: &mut schemars::SchemaGenerator) -> HashMap<String, InputDescription> {
    let mut year_begin_schema = generator.root_schema_for::<Year>().to_value();
    year_begin_schema["minimum"] = serde_json::json!(FUTURE_START_YEAR);
    year_begin_schema["maximum"] = serde_json::json!(FUTURE_END_YEAR);
    year_begin_schema["default"] = serde_json::json!(default_year_begin().0);

    let mut reference_year_begin_schema = generator.root_schema_for::<Year>().to_value();
    reference_year_begin_schema["minimum"] = serde_json::json!(HISTORICAL_START_YEAR);
    reference_year_begin_schema["maximum"] = serde_json::json!(HISTORICAL_END_YEAR);
    reference_year_begin_schema["default"] = serde_json::json!(default_reference_year_begin().0);

    HashMap::from([
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
                schema: year_begin_schema,
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
                        "First year of the reference period used to compute anomalies. Uses the same range as the analysis window.".to_string(),
                    ),
                    ..Default::default()
                },
                schema: reference_year_begin_schema,
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
                    description: Some("The climate-model workflows to execute for each requested variable. If empty, all available models are used.".to_string()),
                    ..Default::default()
                },
                schema: models_schema(generator),
                min_occurs: Some(0),
                ..Default::default()
            },
        ),
    ])
}

fn build_outputs(generator: &mut schemars::SchemaGenerator) -> HashMap<String, OutputDescription> {
    // The registry is the single source of truth for which scenarios can be produced: advertising a
    // scenario no registered model supports would leave the client sending an output that is then
    // dropped (see `resolve_requests`), i.e. a control that silently does nothing.
    let registry: Vec<ClimateModelProperties> = MODEL_REGISTRY.values().cloned().collect();
    let available_scenarios = resolve_available_scenarios(&registry);
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
                    ..Default::default()
                },
                schema: generator
                    .root_schema_for::<DataResource<Vec<ClimateRiskRawRow>>>()
                    .to_value(),
            },
        ),
    ]);

    for props in available_scenarios {
        outputs.insert(
            props.scenario.name().to_string(),
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

    outputs
}

#[async_trait::async_trait]
impl Processor for ClimateRiskProcess {
    type Input = ClimateRiskProcessParams;
    type Output = ClimateRiskOutputs;

    fn id(&self) -> &'static str {
        Self::ID
    }

    fn version(&self) -> &'static str {
        "0.2.0"
    }

    async fn process(&self) -> Result<Process> {
        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;
        let mut generator = settings.into_generator();

        let inputs = build_inputs(&mut generator);
        let outputs = build_outputs(&mut generator);

        Ok(Process {
            summary: ProcessSummary {
                id: self.id().into(),
                version: self.version().into(),
                description: DescriptionType {
                    title: Some("Climate risk indicators".to_string()),
                    description: Some(
                        "This process derives climate-risk indicators such as heat days from NEX-GDDP-CMIP6 climate data for a point location and a time window. The workflow builds a daily threshold mask, aggregates it over the requested years and returns summary statistics for the selected climate variable. An anomaly relative to a historical reference period (same length) is reported as the difference of the multi-year means. If no models are specified, all available models are used. If no scenario outputs are requested, all future scenarios are computed."
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

    async fn parse(&self, execute: Execute) -> Result<Self::Input> {
        Ok(ClimateRiskProcessParams {
            inputs: parse_inputs(&execute.inputs)?,
            requested_outputs: execute.outputs,
        })
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let mut inputs = input.inputs;
        let requested_outputs = input.requested_outputs;

        validate_inputs(
            inputs.year_begin,
            inputs.year_range,
            inputs.reference_year_begin,
        )?;

        let point = inputs.coordinate.value.coordinates.clone();

        let (filtered_models, model_props, dropped_models) = resolve_models(&inputs.models);
        if !dropped_models.is_empty() {
            tracing::warn!(
                "Ignoring unknown climate models: {}",
                dropped_models.join(", ")
            );
        }
        if model_props.is_empty() {
            let detail = if !dropped_models.is_empty() {
                format!(
                    "; none of the requested models ({}) are available",
                    dropped_models.join(", ")
                )
            } else if let Some(problem) = registry_problem() {
                format!("; {problem}")
            } else {
                "; the model registry is empty".to_string()
            };
            anyhow::bail!("No climate models valid / available{detail}");
        }
        inputs.models = filtered_models;

        let scenario_props = resolve_available_scenarios(&model_props);
        if scenario_props.is_empty() {
            anyhow::bail!("No climate scenarios valid / available for the specified models.");
        }

        let available_scenarios: Vec<ClimateScenario> =
            scenario_props.iter().map(|s| s.scenario).collect();

        let output_keys: std::collections::BTreeSet<String> =
            requested_outputs.keys().cloned().collect();
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

        let configuration = CONFIG.geoengine.api_config(CONTEXT.session_token().ok());
        let (mut outputs, computation_ids) = compute_climate(
            &configuration,
            &point,
            &ClimatePeriods {
                analysis_start: inputs.year_begin,
                range: inputs.year_range,
                reference_start: inputs.reference_year_begin,
            },
            &request_props,
            &model_props,
        )
        .await?;

        if should_reflect_inputs {
            outputs.inputs = Some(inputs);
        }
        if !include_raw_ensemble {
            outputs.raw_ensemble_data = None;
        }

        for computation_id in computation_ids.iter().filter(|id| id.is_some()) {
            add_credits_used_pending(self.db.clone(), configuration.clone(), *computation_id)
                .await?;
        }

        Ok(outputs)
    }
}

/// Parsed and validated [`Execute`] payload for the climate-risk process.
pub struct ClimateRiskProcessParams {
    pub inputs: ClimateRiskInputs,
    pub requested_outputs: HashMap<String, ogcapi::types::processes::Output>,
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
    Year(reference_year): Year,
) -> Result<()> {
    if !(5..=30).contains(&range) {
        anyhow::bail!("Year range must be between 5 and 30 years");
    }
    if start_year < FUTURE_START_YEAR {
        anyhow::bail!("Start year must be at least {FUTURE_START_YEAR}");
    }
    if start_year + range > FUTURE_END_YEAR {
        anyhow::bail!("Start year plus range must not exceed {FUTURE_END_YEAR}");
    }
    if reference_year < HISTORICAL_START_YEAR {
        anyhow::bail!("Reference period start year must be at least {HISTORICAL_START_YEAR}");
    }
    if reference_year + range > HISTORICAL_END_YEAR {
        anyhow::bail!(
            "Reference period start year plus range must not exceed {HISTORICAL_END_YEAR}"
        );
    }
    Ok(())
}

/// Filters the requested models down to those in the registry. The third return value
/// lists the user-specified models that were dropped, so callers can report them.
fn resolve_models(
    specified_models: &[String],
) -> (Vec<String>, Vec<ClimateModelProperties>, Vec<String>) {
    if specified_models.is_empty() {
        let (models, props): (Vec<_>, Vec<_>) = MODEL_REGISTRY
            .iter()
            .map(|(id, props)| (id.clone(), props.clone()))
            .unzip();
        (models, props, Vec::new())
    } else {
        let mut models = Vec::new();
        let mut props = Vec::new();
        let mut dropped = Vec::new();
        for model_id in specified_models {
            if let Some(model_props) = MODEL_REGISTRY.get(model_id) {
                models.push(model_id.clone());
                props.push(model_props.clone());
            } else {
                dropped.push(model_id.clone());
            }
        }
        (models, props, dropped)
    }
}

fn resolve_available_scenarios(
    model_props: &[ClimateModelProperties],
) -> Vec<ClimateScenarioProperties> {
    ClimateScenario::FUTURE
        .iter()
        .copied()
        .filter(|s| model_props.iter().any(|m| m.scenarios.contains(s)))
        .map(ClimateScenario::properties)
        .collect()
}

fn resolve_variables(specified_variables: &[ClimateVariable]) -> Vec<ClimateVariable> {
    let mut seen = std::collections::HashSet::new();
    let mut variables = Vec::new();
    for variable in specified_variables {
        if seen.insert(*variable) {
            variables.push(*variable);
        }
    }
    if variables.is_empty() {
        ClimateVariable::ALL.to_vec()
    } else {
        variables
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

        let future = ClimateScenario::FUTURE
            .iter()
            .copied()
            .find(|scenario| scenario.name() == output_key);
        match future {
            // A future scenario no selected model supports is skipped, not an error: the requested
            // models can be narrower than the registry, and the client only ever sees the scenarios
            // `build_outputs` advertised. Anything else (`historical`, a typo) stays an error.
            Some(scenario) if !available_scenarios.contains(&scenario) => {
                tracing::warn!(
                    "Skipping output `{output_key}`: no selected model provides that scenario"
                );
            }
            Some(scenario) => selected_scenarios.push(scenario),
            None => anyhow::bail!("Unknown output requested: {output_key}"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use geoengine_api_client::models::RasterOperator;
    use ogcapi::types::processes::Input;
    use serde_json::json;

    #[test]
    fn it_deserializes_the_input() {
        let payload = json!({
            "coordinate": {
                "value": {
                    "type": "Point",
                    "coordinates": [12.34, 56.78]
                },
                "mediaType": "application/geo+json"
            },
            "yearBegin": 2050,
            "yearRange": 30,
            "referenceYearBegin": 1981,
            "variables": ["heatDays", "iceDays"],
            "models": ["ACCESS-CM2"]
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(payload).unwrap();
        let inputs = parse_inputs(&inputs).unwrap();
        assert_eq!(
            inputs.variables,
            vec![ClimateVariable::HeatDays, ClimateVariable::IceDays]
        );
        assert_eq!(inputs.reference_year_begin, Year(1981));
    }

    #[test]
    fn it_deserializes_omitted_optional_inputs_as_their_defaults() {
        let payload = json!({
            "coordinate": {
                "value": {
                    "type": "Point",
                    "coordinates": [12.34, 56.78]
                },
                "mediaType": "application/geo+json"
            },
            "referenceYearBegin": 1981
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(payload).unwrap();
        let inputs = parse_inputs(&inputs).unwrap();
        assert_eq!(inputs.reference_year_begin, Year(1981));
    }

    #[test]
    fn it_rejects_missing_reference_year_begin() {
        let payload = json!({
            "coordinate": {
                "value": {
                    "type": "Point",
                    "coordinates": [12.34, 56.78]
                },
                "mediaType": "application/geo+json"
            }
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(payload).unwrap();
        let error = format!("{:#}", parse_inputs(&inputs).unwrap_err());

        assert!(error.contains("referenceYearBegin"), "{error}");
    }

    #[test]
    fn it_rejects_malformed_inputs_with_context() {
        let payload = json!({ "coordinate": { "value": { "type": "Point" }, "mediaType": "application/geo+json" } });

        let inputs: HashMap<String, Input> = serde_json::from_value(payload).unwrap();
        let error = format!("{:#}", parse_inputs(&inputs).unwrap_err());

        assert!(
            error.contains("Failed to deserialize climate-risk inputs"),
            "{error}"
        );
        assert!(error.contains("coordinate"), "{error}");
    }

    #[crate::test]
    async fn it_declares_expected_inputs_and_outputs_in_the_summary(db: DbHandle) {
        let process = ClimateRiskProcess::new(db).process().await.unwrap();

        assert_eq!(process.summary.id, "climate-risk");
        assert_eq!(process.summary.version, "0.2.0");

        assert!(!process.inputs.contains_key("scenarios"));
        assert!(!process.inputs.contains_key("yearEnd"));
        assert!(!process.inputs.contains_key("region"));
        assert!(process.inputs.contains_key("variables"));
        assert!(process.inputs.contains_key("yearRange"));
        assert!(process.inputs.contains_key("referenceYearBegin"));

        assert!(process.outputs.contains_key("rawEnsembleData"));
        assert!(!process.outputs.contains_key("historical"));
        assert!(!process.outputs.contains_key("climateRisk"));
        assert_eq!(
            process.outputs["ssp245"].description_type.title.as_deref(),
            Some("SSP2-4.5 (Intermediate emissions)")
        );

        // Declared scenario outputs track the registry, so no registered scenario is left out and no
        // unavailable one is offered.
        let registry: Vec<ClimateModelProperties> = MODEL_REGISTRY.values().cloned().collect();
        let scenario_keys: Vec<String> = ClimateScenario::FUTURE
            .iter()
            .map(|scenario| scenario.name().to_string())
            .collect();
        let declared_scenarios: std::collections::BTreeSet<String> = process
            .outputs
            .keys()
            .filter(|key| scenario_keys.contains(key))
            .cloned()
            .collect();
        let expected: std::collections::BTreeSet<String> = resolve_available_scenarios(&registry)
            .iter()
            .map(|props| props.scenario.name().to_string())
            .collect();
        assert_eq!(declared_scenarios, expected);
    }

    #[crate::test]
    async fn it_marks_reference_year_begin_as_required_with_a_default(db: DbHandle) {
        let process = ClimateRiskProcess::new(db).process().await.unwrap();
        let input = &process.inputs["referenceYearBegin"];

        assert_eq!(input.schema["type"], json!("integer"));
        assert!(input.schema.get("anyOf").is_none());
        assert_eq!(input.schema["default"], json!(1981));
        assert_eq!(
            input.description_type.metadata.len(),
            0,
            "no metadata should be present: {:#?}",
            input.description_type.metadata
        );
        assert_eq!(input.min_occurs.unwrap_or(1), 1);
    }

    #[test]
    fn it_validates_year_ranges() {
        let cases: &[(Year, YearRange, Year, bool)] = &[
            (Year(2050), YearRange(4), Year(1981), false), // range too small
            (Year(2050), YearRange(31), Year(1981), false), // range too large
            (Year(2080), YearRange(30), Year(1981), false), // analysis end > 2100
            (Year(2050), YearRange(20), Year(1949), false), // reference start < 1950
            (Year(2014), YearRange(20), Year(1981), false), // analysis start < 2015
            (Year(2050), YearRange(20), Year(1981), true), // valid
            (Year(2050), YearRange(20), Year(2015), false), // reference end > 2014
            (Year(2050), YearRange(5), Year(1981), true),  // valid, min range
            (Year(2050), YearRange(30), Year(1981), true), // valid, max range
        ];
        for &(start, range, reference, expected_ok) in cases {
            let ok = validate_inputs(start, range, reference).is_ok();
            assert_eq!(
                ok, expected_ok,
                "validate_inputs({start:?}, {range:?}, {reference:?})"
            );
        }
    }

    #[test]
    fn it_names_dataset_raster_sources() {
        let scenario = ClimateScenario::Ssp245.properties();
        let model = ClimateModelProperties {
            id: "ACCESS-CM2".to_string(),
            variant: "r1i1p1f1".to_string(),
            grid: "gn".to_string(),
            scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
        };
        let var = ClimateVariable::HeatDays.properties();

        let result = ClimateRiskProcess::dataset_raster_source(&var, &model, &scenario);

        assert!(matches!(result, RasterOperator::GdalSource(_)));

        let value = serde_json::to_value(&result).unwrap();
        assert_eq!(
            value["params"]["data"],
            "nexgddp_cmip6_ACCESS-CM2_ssp245_tasmax"
        );
    }

    #[test]
    fn it_resolves_models() {
        let (models, _props, dropped) = resolve_models(&[]);
        assert!(!models.is_empty());
        assert!(dropped.is_empty());

        let first_model = models[0].clone();
        let (models, _props, dropped) = resolve_models(std::slice::from_ref(&first_model));
        assert_eq!(models, vec![first_model]);
        assert!(dropped.is_empty());

        let (_, _, dropped) = resolve_models(&["nonexistent-model".to_string()]);
        assert_eq!(dropped, vec!["nonexistent-model".to_string()]);
    }

    #[test]
    fn it_resolves_available_scenarios() {
        let model_props: Vec<ClimateModelProperties> = MODEL_REGISTRY.values().cloned().collect();
        let scenarios = resolve_available_scenarios(&model_props);
        assert!(!scenarios.is_empty());
        assert!(resolve_available_scenarios(&[]).is_empty());
    }

    #[test]
    fn it_resolves_variables() {
        assert_eq!(resolve_variables(&[]), ClimateVariable::ALL.to_vec());
        assert_eq!(
            resolve_variables(&[ClimateVariable::HeatDays]),
            vec![ClimateVariable::HeatDays]
        );
        assert_eq!(
            resolve_variables(&[
                ClimateVariable::HeatDays,
                ClimateVariable::HeatDays,
                ClimateVariable::IceDays
            ]),
            vec![ClimateVariable::HeatDays, ClimateVariable::IceDays]
        );
    }

    #[test]
    fn it_enumerates_the_registered_models_in_the_models_schema() {
        let mut settings = schemars::generate::SchemaSettings::default();
        settings.meta_schema = None;
        let mut generator = settings.into_generator();
        let schema = models_schema(&mut generator);
        let items = schema["items"].as_object().unwrap();
        let values = items["enum"].as_array().unwrap();
        assert_eq!(values.len(), MODEL_REGISTRY.len());
        for id in MODEL_REGISTRY.keys() {
            assert!(
                values.contains(&serde_json::Value::String(id.clone())),
                "{id} missing from models enum"
            );
        }
    }

    #[test]
    fn it_resolves_requests() {
        let scenarios = vec![ClimateScenario::Ssp245, ClimateScenario::Ssp585];

        let (selected, should_reflect, include_raw) =
            resolve_requests(&std::collections::BTreeSet::new(), &scenarios).unwrap();
        assert_eq!(selected, scenarios);
        assert!(should_reflect);
        assert!(!include_raw);

        let keys = std::collections::BTreeSet::from(["ssp245".to_string(), "ssp585".to_string()]);
        let (selected, should_reflect, _) = resolve_requests(&keys, &scenarios).unwrap();
        assert_eq!(selected, scenarios);
        assert!(!should_reflect);

        let keys = std::collections::BTreeSet::from(["inputs".to_string()]);
        let (_, should_reflect, include_raw) = resolve_requests(&keys, &scenarios).unwrap();
        assert!(should_reflect);
        assert!(!include_raw);

        let keys = std::collections::BTreeSet::from(["rawEnsembleData".to_string()]);
        let (_, should_reflect, include_raw) = resolve_requests(&keys, &scenarios).unwrap();
        assert!(!should_reflect);
        assert!(include_raw);
    }

    #[test]
    fn it_skips_scenarios_the_selected_models_do_not_provide() {
        // Only ssp245 is available: asking for ssp585 must skip it, not fail the whole run.
        let available = vec![ClimateScenario::Ssp245];
        let keys = std::collections::BTreeSet::from(["ssp245".to_string(), "ssp585".to_string()]);
        let (selected, _, _) = resolve_requests(&keys, &available).unwrap();
        assert_eq!(selected, vec![ClimateScenario::Ssp245]);

        // Skipping the only requested scenario falls back to everything available.
        let keys = std::collections::BTreeSet::from(["ssp585".to_string()]);
        let (selected, _, _) = resolve_requests(&keys, &available).unwrap();
        assert_eq!(selected, available);
    }

    #[test]
    fn it_rejects_invalid_requests() {
        let scenarios = vec![ClimateScenario::Ssp245];
        // `historical` is never an output, so it must stay an error rather than a silent skip.
        for key in ["nonexistent", "climateRisk", "historical"] {
            let keys = std::collections::BTreeSet::from([key.to_string()]);
            assert!(
                resolve_requests(&keys, &scenarios).is_err(),
                "expected error for key {key}"
            );
        }
    }
}
