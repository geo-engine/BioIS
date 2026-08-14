use super::*;
use crate::processes::parameters::{BioisDisplayKind, BoundingBox, DataResource, TableSchemaType};
use crate::profile::CLIMATE_RISK_TABLE_SCHEMA_PROFILE;
use approx::assert_abs_diff_eq;
use geoengine_api_client::models::{
    CollectionType, GeoJson, RasterDataType, RasterOperator, VectorOperator,
};
use ogcapi::types::processes::{InlineOrRefData, Input};
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
        "yearBegin": 2014,
        "yearRange": 20,
        "referenceYearBegin": 2020,
        "variables": ["heatDays", "iceDays"],
        "models": ["MPI-M-MPI-ESM-LR"],
        "region": "Eur"
    });

    let inputs: HashMap<String, Input> = serde_json::from_value(payload).unwrap();
    let inputs = parse_inputs(&inputs).unwrap();
    assert_eq!(
        inputs.variables,
        vec![ClimateVariable::HeatDays, ClimateVariable::IceDays]
    );
    assert_eq!(inputs.reference_year_begin, Year(2020));
    assert_eq!(inputs.region, Some(CordexRegion::Eur));
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
        "referenceYearBegin": 2020
    });

    let inputs: HashMap<String, Input> = serde_json::from_value(payload).unwrap();
    let inputs = parse_inputs(&inputs).unwrap();
    assert_eq!(inputs.reference_year_begin, Year(2020));
    assert_eq!(inputs.region, None);
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

#[test]
fn it_process_summary_has_expected_inputs_and_outputs() {
    let process = ClimateRiskProcess.process().unwrap();

    assert_eq!(process.summary.id, "climate-risk");
    assert_eq!(process.summary.version, "0.2.0");

    assert!(!process.inputs.contains_key("scenarios"));
    assert!(!process.inputs.contains_key("yearEnd"));
    assert!(process.inputs.contains_key("variables"));
    assert!(process.inputs.contains_key("yearRange"));
    assert!(process.inputs.contains_key("referenceYearBegin"));

    assert!(process.outputs.contains_key("rcp26"));
    assert!(process.outputs.contains_key("rcp45"));
    assert!(process.outputs.contains_key("rcp85"));
    assert!(process.outputs.contains_key("rawEnsembleData"));
    assert!(!process.outputs.contains_key("climateRisk"));
    assert_eq!(
        process.outputs["rcp45"].description_type.title.as_deref(),
        Some("RCP 4.5 (Intermediate emissions)")
    );
    assert_eq!(
        process.outputs["rawEnsembleData"]
            .description_type
            .metadata
            .first()
            .and_then(|m| m.role.as_deref()),
        Some("default-disabled")
    );
}

#[test]
fn it_reference_year_begin_schema_is_required_with_default() {
    let process = ClimateRiskProcess.process().unwrap();
    let input = &process.inputs["referenceYearBegin"];

    assert_eq!(input.schema["type"], json!("integer"));
    assert!(input.schema.get("anyOf").is_none());
    assert_eq!(input.schema["default"], json!(2020));
    assert_eq!(
        input.description_type.metadata.len(),
        0,
        "no metadata should be present: {:#?}",
        input.description_type.metadata
    );
    assert_eq!(input.min_occurs.unwrap_or(1), 1);
}

#[test]
fn it_validate_inputs_rejects_range_below_min() {
    assert!(validate_inputs(Year(2014), YearRange(4), Year(2020)).is_err());
}

#[test]
fn it_validate_inputs_rejects_range_above_max() {
    assert!(validate_inputs(Year(2014), YearRange(31), Year(2020)).is_err());
}

#[test]
fn it_validate_inputs_rejects_range_beyond_2100() {
    assert!(validate_inputs(Year(2080), YearRange(30), Year(2020)).is_err());
}

#[test]
fn it_validate_inputs_rejects_reference_before_data_start() {
    assert!(validate_inputs(Year(2014), YearRange(20), Year(2005)).is_err());
}

#[test]
fn it_validate_inputs_rejects_start_year_before_data_start() {
    assert!(validate_inputs(Year(2005), YearRange(20), Year(2020)).is_err());
    assert!(validate_inputs(Year(2006), YearRange(20), Year(2020)).is_ok());
}

#[test]
fn it_validate_inputs_rejects_reference_beyond_2100() {
    assert!(validate_inputs(Year(2014), YearRange(20), Year(2090)).is_err());
}

#[test]
fn it_validate_inputs_accepts_valid_range() {
    assert!(validate_inputs(Year(2014), YearRange(5), Year(2020)).is_ok());
    assert!(validate_inputs(Year(2014), YearRange(30), Year(2020)).is_ok());
}

#[test]
fn it_climate_variable_props_values() {
    for (var, expected_name, expected_suffix, expected_expr) in [
        (ClimateVariable::HeatDays, "Heat Days", "tasmax", "c >= 30"),
        (ClimateVariable::IceDays, "Ice Days", "tasmax", "c < 0"),
        (
            ClimateVariable::TropicalNights,
            "Tropical Nights",
            "tasmin",
            "c > 20",
        ),
        (ClimateVariable::FrostDays, "Frost Days", "tasmin", "c < 0"),
        (ClimateVariable::DryDays, "Dry Days", "pr", "86400) < 1"),
        (
            ClimateVariable::HeavyRainDays,
            "Heavy Rain Days",
            "pr",
            "86400) > 20",
        ),
    ] {
        let props = var.properties();
        assert_eq!(props.name, expected_name);
        assert_eq!(props.dataset_variable_suffix, expected_suffix);
        assert!(props.expression.contains(expected_expr));
    }
}

#[test]
fn it_cordex_model_props_values() {
    for (model, expected_name, expected_prefix) in [
        (
            CordexModel::MpiMmpiEsmLr,
            "MPI-M-MPI-ESM-LR",
            "MPI-M-MPI-ESM-LR",
        ),
        (
            CordexModel::MohcHadgem2Es,
            "MOHC-HadGEM2-ES",
            "MOHC-HadGEM2-ES",
        ),
    ] {
        let props = model.properties();
        assert_eq!(props.model, model);
        assert_eq!(props.name, expected_name);
        assert_eq!(props.dataset_prefix, expected_prefix);
        assert_eq!(props.region, CordexRegion::Eur);
        assert_eq!(
            props.scenarios,
            vec![
                ClimateScenario::Rcp26,
                ClimateScenario::Rcp45,
                ClimateScenario::Rcp85
            ]
        );
    }
}

#[test]
fn it_climate_scenario_props_values() {
    for (scenario, expected_name, expected_prefix) in [
        (ClimateScenario::Rcp26, "RCP 2.6 (Low emissions)", "rcp26"),
        (
            ClimateScenario::Rcp45,
            "RCP 4.5 (Intermediate emissions)",
            "rcp45",
        ),
        (ClimateScenario::Rcp85, "RCP 8.5 (High emissions)", "rcp85"),
    ] {
        let props = scenario.properties();
        assert_eq!(props.scenario, scenario);
        assert_eq!(props.name, expected_name);
        assert_eq!(props.dataset_prefix, expected_prefix);
    }
}

#[test]
fn it_cordex_region_props_values() {
    let props = CordexRegion::Eur.properties();
    assert_eq!(props.region, CordexRegion::Eur);
    assert_eq!(props.name, "Europe");
    assert_eq!(props.dataset_prefix, "EUR11");
    assert_eq!(props.bounding_box.wfs_string(), "-10,34,30,72");
}

#[test]
fn it_point_to_region_matches_inside_bbox() {
    let point = PointType::from(vec![12.34, 56.78]);
    assert_eq!(
        CordexRegion::point_to_region(&point),
        Some(CordexRegion::Eur)
    );
}

#[test]
fn it_point_to_region_returns_none_outside() {
    let point = PointType::from(vec![0.0, 0.0]);
    assert_eq!(CordexRegion::point_to_region(&point), None);
}

#[test]
fn it_bounding_box_around_point() {
    let point = PointType::from(vec![10.0, 50.0]);
    let bbox = BoundingBox::around_point(&point, 0.0001);
    assert_eq!(bbox.wfs_string(), "9.9999,49.9999,10.0001,50.0001");
    assert!(bbox.contains(&point));
}

#[test]
fn it_dataset_raster_source_naming() {
    let region = CordexRegion::Eur.properties();
    let scenario = ClimateScenario::Rcp45.properties();
    let model = CordexModel::MpiMmpiEsmLr.properties();
    let var = ClimateVariable::HeatDays.properties();

    let result = ClimateRiskProcess::dataset_raster_source(&var, &model, &scenario, &region);

    assert!(matches!(result, RasterOperator::GdalSource(_)));

    let value = serde_json::to_value(&result).unwrap();
    assert_eq!(
        value["params"]["data"],
        "cordex_EUR11_rcp45_MPI-M-MPI-ESM-LR_tasmax"
    );
}

#[test]
fn it_resolves_region_explicit_valid() {
    let point = PointType::from(vec![12.0, 50.0]);
    let result = resolve_region(Some(CordexRegion::Eur), &point);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().region, CordexRegion::Eur);
}

#[test]
fn it_resolves_region_explicit_invalid() {
    let point = PointType::from(vec![0.0, 0.0]);
    let result = resolve_region(Some(CordexRegion::Eur), &point);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("outside of the specified")
    );
}

#[test]
fn it_resolves_region_inferred_inside() {
    let point = PointType::from(vec![12.0, 50.0]);
    let result = resolve_region(None, &point);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().region, CordexRegion::Eur);
}

#[test]
fn it_resolves_region_inferred_outside() {
    let point = PointType::from(vec![0.0, 0.0]);
    let result = resolve_region(None, &point);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("outside of the supported")
    );
}

#[test]
fn it_resolves_models_all_when_empty() {
    let (models, props, dropped) = resolve_models(&[], CordexRegion::Eur);
    assert_eq!(models.len(), 2);
    assert_eq!(props.len(), 2);
    assert!(dropped.is_empty());
    assert!(models.contains(&CordexModel::MpiMmpiEsmLr));
    assert!(models.contains(&CordexModel::MohcHadgem2Es));
}

#[test]
fn it_resolves_models_filtered() {
    let (models, props, dropped) = resolve_models(&[CordexModel::MpiMmpiEsmLr], CordexRegion::Eur);
    assert_eq!(models.len(), 1);
    assert_eq!(props.len(), 1);
    assert!(dropped.is_empty());
    assert_eq!(models[0], CordexModel::MpiMmpiEsmLr);
    assert_eq!(props[0].name, "MPI-M-MPI-ESM-LR");
}

#[test]
fn it_resolves_available_scenarios_returns_all_supported() {
    let model_props = vec![CordexModel::MpiMmpiEsmLr.properties()];
    let result = resolve_available_scenarios(&model_props);
    assert_eq!(result.len(), ClimateScenario::ALL.len());
}

#[test]
fn it_resolves_available_scenarios_empty_when_no_models() {
    let result = resolve_available_scenarios(&[]);
    assert!(result.is_empty());
}

#[test]
fn it_resolves_variables_empty_means_all() {
    assert_eq!(resolve_variables(&[]), ClimateVariable::ALL.to_vec());
    assert_eq!(
        resolve_variables(&[ClimateVariable::HeatDays]),
        vec![ClimateVariable::HeatDays]
    );
}

#[test]
fn it_resolves_requests_empty_means_all_and_reflect() {
    let keys = std::collections::BTreeSet::new();
    let scenarios = vec![ClimateScenario::Rcp45];
    let (selected, should_reflect, include_raw_ensemble) =
        resolve_requests(&keys, &scenarios).unwrap();
    assert_eq!(selected, vec![ClimateScenario::Rcp45]);
    assert!(should_reflect);
    assert!(!include_raw_ensemble);
}

#[test]
fn it_resolves_requests_selects_scenarios() {
    let mut keys = std::collections::BTreeSet::new();
    keys.insert("rcp45".to_string());
    keys.insert("rcp85".to_string());
    let scenarios = vec![ClimateScenario::Rcp45, ClimateScenario::Rcp85];
    let (selected, should_reflect, _) = resolve_requests(&keys, &scenarios).unwrap();
    assert_eq!(
        selected,
        vec![ClimateScenario::Rcp45, ClimateScenario::Rcp85]
    );
    assert!(!should_reflect);
}

#[test]
fn it_rejects_unknown_requests() {
    let mut keys = std::collections::BTreeSet::new();
    keys.insert("nonexistent".to_string());
    let scenarios = vec![ClimateScenario::Rcp45];
    assert!(resolve_requests(&keys, &scenarios).is_err());
}

#[test]
fn it_rejects_legacy_climate_risk_key() {
    let mut keys = std::collections::BTreeSet::new();
    keys.insert("climateRisk".to_string());
    let scenarios = vec![ClimateScenario::Rcp45];
    assert!(resolve_requests(&keys, &scenarios).is_err());
}

#[test]
fn it_resolves_input_output_defaults() {
    let mut keys = std::collections::BTreeSet::new();
    keys.insert("inputs".to_string());
    let scenarios = vec![ClimateScenario::Rcp45];
    let (selected, should_reflect, include_raw_ensemble) =
        resolve_requests(&keys, &scenarios).unwrap();
    assert_eq!(selected, vec![ClimateScenario::Rcp45]);
    assert!(should_reflect);
    assert!(!include_raw_ensemble);
}

#[test]
fn it_resolves_raw_ensemble_output_defaults() {
    let mut keys = std::collections::BTreeSet::new();
    keys.insert("rawEnsembleData".to_string());
    let scenarios = vec![ClimateScenario::Rcp45];
    let (selected, should_reflect, include_raw_ensemble) =
        resolve_requests(&keys, &scenarios).unwrap();
    assert_eq!(selected, vec![ClimateScenario::Rcp45]);
    assert!(!should_reflect);
    assert!(include_raw_ensemble);
}

#[test]
fn it_execute_results_group_rows_by_scenario() {
    fn row(scenario: &str) -> ClimateRiskRow {
        ClimateRiskRow {
            variable: "Heat Days".to_string(),
            scenario: scenario.to_string(),
            mean: 1.0,
            median: 1.0,
            min: 1.0,
            max: 1.0,
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
    assert!(result.contains_key("RCP 2.6 (Low emissions)"));
    assert!(result.contains_key("RCP 4.5 (Intermediate emissions)"));
    assert!(!result.contains_key("RCP 8.5 (High emissions)"));
    assert!(!result.contains_key("rcp26"));

    let InlineOrRefData::QualifiedInputValue(qualified) = &result["RCP 2.6 (Low emissions)"].data
    else {
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
fn it_aggregate_from_list_aggregates_values() {
    let mut values: HashMap<CordexModel, f64> = HashMap::new();
    values.insert(CordexModel::MpiMmpiEsmLr, 10.0);
    values.insert(CordexModel::MohcHadgem2Es, 20.0);

    let result = aggregate_from_list(&values).unwrap();
    assert_abs_diff_eq!(result.min, 10.0);
    assert_abs_diff_eq!(result.max, 20.0);
    assert_abs_diff_eq!(result.mean, 15.0);
    assert_abs_diff_eq!(result.median, 15.0);
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
        max: 100.0,
        min: 0.0,
        mean: 50.0,
        median: 50.0,
        occurrence_probability: Some(0.5),
        anomaly: Some(10.0),
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
            max: 100.0,
            min: 0.0,
            mean: 50.0,
            median: 50.0,
            occurrence_probability: Some(0.5),
            anomaly: Some(10.0),
            ..Default::default()
        },
        ClimateRiskRow {
            variable: "Dry Days".to_string(),
            scenario: "rcp45".to_string(),
            max: 100.0,
            min: 0.0,
            mean: 50.0,
            median: 50.0,
            occurrence_probability: Some(0.5),
            anomaly: Some(5.0),
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
        max: 100.0,
        min: 0.0,
        mean: 50.0,
        median: 50.0,
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
        max: 100.0,
        min: 0.0,
        mean: 50.0,
        median: 50.0,
        occurrence_probability: Some(0.5),
        anomaly: Some(10.0),
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
            max: 100.0,
            min: 0.0,
            mean: 50.0,
            median: 50.0,
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
fn it_climate_variable_properties_accessors() {
    let props = ClimateVariable::HeatDays.properties();
    assert_eq!(props.name_string(), "Heat Days");
    assert_eq!(ClimateVariableProperties::measurement_unit(), "Days");
    assert_eq!(ClimateVariableProperties::measurement_string(), "Days");
    assert_eq!(
        ClimateVariableProperties::expression_dtype(),
        RasterDataType::I8
    );
    assert_eq!(
        ClimateVariableProperties::year_agg_dtype(),
        RasterDataType::U16
    );
}

#[test]
fn it_outputs_to_execute_results() {
    let rows = vec![ClimateRiskRow {
        variable: "Heat Days".to_string(),
        scenario: "rcp45".to_string(),
        max: 100.0,
        min: 0.0,
        mean: 50.0,
        median: 50.0,
        occurrence_probability: None,
        anomaly: Some(10.0),
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
fn it_outputs_from_feature_collection_ok() {
    let geo_json = GeoJson {
        features: vec![
            serde_json::json!({
                "type": "Feature",
                "properties": {
                    "MPI-M-MPI-ESM-LR": 42.0,
                    "MOHC-HadGEM2-ES": 10.0
                }
            }),
            serde_json::json!({
                "type": "Feature",
                "properties": {
                    "MPI-M-MPI-ESM-LR": 58.0,
                    "MOHC-HadGEM2-ES": 30.0
                }
            }),
        ],
        r#type: CollectionType::FeatureCollection,
    };

    let models = vec![
        CordexModel::MpiMmpiEsmLr.properties(),
        CordexModel::MohcHadgem2Es.properties(),
    ];

    let result = outputs_from_feature_collection(&geo_json, &models).unwrap();
    assert_eq!(result.len(), 2);
    assert_abs_diff_eq!(result[&CordexModel::MpiMmpiEsmLr], 50.0);
    assert_abs_diff_eq!(result[&CordexModel::MohcHadgem2Es], 20.0);
}

#[test]
fn it_outputs_from_feature_collection_empty() {
    let geo_json = GeoJson::default();
    let models = vec![CordexModel::MpiMmpiEsmLr.properties()];
    assert!(outputs_from_feature_collection(&geo_json, &models).is_err());
}

#[test]
fn it_outputs_from_feature_collection_no_properties() {
    let geo_json = GeoJson {
        features: vec![serde_json::json!({ "type": "Feature" })],
        r#type: CollectionType::FeatureCollection,
    };
    let models = vec![CordexModel::MpiMmpiEsmLr.properties()];
    assert!(outputs_from_feature_collection(&geo_json, &models).is_err());
}

#[test]
fn it_vector_source_creates_mock_point_source() {
    let point = PointType::from(vec![12.0, 34.0]);
    let result = vector_source(&point);
    assert!(matches!(result, VectorOperator::MockPointSource(_)));
}

#[test]
fn it_build_variable_workflows_chain() {
    let region = CordexRegion::Eur.properties();
    let scenario = ClimateScenario::Rcp45.properties();
    let model = CordexModel::MpiMmpiEsmLr.properties();
    let var = ClimateVariable::HeatDays.properties();

    let day_expr =
        ClimateRiskProcess::build_variable_day_expression(&var, &model, &scenario, &region);
    assert!(matches!(day_expr, RasterOperator::Expression(_)));

    let year_agg =
        ClimateRiskProcess::build_variable_year_agg_workflow(&var, &model, &scenario, &region);
    assert!(matches!(
        year_agg,
        RasterOperator::TemporalRasterAggregation(_)
    ));
}

#[test]
fn it_probability_label_maps_classes() {
    assert_eq!(probability_label(0.0), "1 · extremely low (0 %)");
    assert_eq!(probability_label(0.0001), "1 · extremely low (0 %)");
    assert_eq!(probability_label(0.0005), "2 · very low (0.1 %)");
    assert_eq!(probability_label(0.001), "3 · low (0.1 %)");
    assert_eq!(probability_label(0.005), "5 · moderate (0.5 %)");
    assert_eq!(probability_label(0.02), "7 · high (2 %)");
    assert_eq!(probability_label(0.03), "7 · high (3 %)");
    assert_eq!(probability_label(0.1), "9 · extremely high (10 %)");
    assert_eq!(probability_label(0.2), "10 · extreme (20 %)");
    assert_eq!(probability_label(0.9), "10 · extreme (90 %)");
}

#[test]
fn it_probability_class_uses_existing_boundaries() {
    let boundaries = [
        (0.00005, ProbabilityClass::ExtremelyLow),
        (0.0002, ProbabilityClass::VeryLow),
        (0.001, ProbabilityClass::Low),
        (0.002, ProbabilityClass::ModeratelyLow),
        (0.004, ProbabilityClass::Moderate),
        (0.01, ProbabilityClass::ModeratelyHigh),
        (0.02, ProbabilityClass::High),
        (0.05, ProbabilityClass::VeryHigh),
        (0.1, ProbabilityClass::ExtremelyHigh),
        (0.2, ProbabilityClass::Extreme),
    ];
    for (probability, expected) in boundaries {
        assert_eq!(ProbabilityClass::from_probability(probability), expected);
    }
    assert_eq!(
        ProbabilityClass::from_probability(0.0),
        ProbabilityClass::ExtremelyLow
    );
    assert_eq!(
        ProbabilityClass::from_probability(1.0),
        ProbabilityClass::Extreme
    );
    assert_eq!(ProbabilityClass::High.return_period_years(), 50);
}

#[test]
fn it_probability_color_maps_classes() {
    assert_eq!(probability_color(0.0), "#66bb6a");
    assert_eq!(probability_color(0.02), "#e53935");
    assert_eq!(probability_color(0.03), "#e53935");
    assert_eq!(probability_color(0.2), "#4a148c");
    assert_eq!(probability_color(0.9), "#4a148c");
}

#[test]
fn it_anomaly_pct_maps_change_and_zero_reference() {
    assert_abs_diff_eq!(anomaly_pct(120.0, 100.0), 20.0);
    assert_abs_diff_eq!(anomaly_pct(50.0, 100.0), -50.0);
    assert_abs_diff_eq!(anomaly_pct(10.0, 0.0), 100.0);
    assert_abs_diff_eq!(anomaly_pct(-10.0, 0.0), -100.0);
    assert_abs_diff_eq!(anomaly_pct(0.0, 0.0), 0.0);
}

#[test]
fn it_anomaly_label_maps_days_and_raw_pct() {
    assert_eq!(anomaly_label(10.0, 20.0), "+10 days (+20 %)");
    assert_eq!(anomaly_label(-5.0, -10.0), "-5 days (-10 %)");
    assert_eq!(anomaly_label(0.0, 0.0), "0 days (0 %)");
    assert_eq!(anomaly_label(10.5, 33.3), "+10.5 days (+33.3 %)");
    assert_eq!(
        anomaly_label(10.0, 250.0),
        "+10 days (+250 %)",
        "label shows the raw percentage, the color clamps separately"
    );
}

#[test]
fn it_percentage_color_maps_and_clamps() {
    assert_eq!(percentage_color(-100.0), "#2166ac");
    assert_eq!(percentage_color(-67.0), "#67a9cf");
    assert_eq!(percentage_color(-33.0), "#d1e5f0");
    assert_eq!(percentage_color(0.0), "#f7f7f7");
    assert_eq!(percentage_color(33.0), "#fddbc7");
    assert_eq!(percentage_color(67.0), "#ef8a62");
    assert_eq!(percentage_color(100.0), "#b2182b");
    assert_eq!(percentage_color(999.0), "#b2182b");
    assert_eq!(percentage_color(-999.0), "#2166ac");
}

#[test]
fn it_climate_risk_row_serializes_display_fields() {
    let row = ClimateRiskRow {
        variable: "Heat Days".to_string(),
        scenario: "rcp45".to_string(),
        mean: 50.0,
        median: 50.0,
        min: 0.0,
        max: 100.0,
        occurrence_probability: Some(0.03),
        anomaly: Some(10.0),
        occurrence_probability_label: Some("7 · high (3 %)".to_string()),
        occurrence_probability_color: Some("#e53935".to_string()),
        anomaly_label: Some("+10 days (+20 %)".to_string()),
        anomaly_color: Some("#fddbc7".to_string()),
    };
    let json = serde_json::to_value(&row).unwrap();
    assert_eq!(json["occurrenceProbabilityLabel"], "7 · high (3 %)");
    assert_eq!(json["occurrenceProbabilityColor"], "#e53935");
    assert_eq!(json["anomalyLabel"], "+10 days (+20 %)");
    assert_eq!(json["anomalyColor"], "#fddbc7");
}
