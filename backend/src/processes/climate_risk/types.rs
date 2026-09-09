use crate::processes::parameters::{
    DataResource, DataResourceSchema, Days, PointGeoJsonInput, Year, YearRange,
};
use geoengine_api_client::models::RasterDataType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Climate variable to compute (WMO-based daily threshold indicators).
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone, PartialEq, Eq, Hash)]
#[schema(title = "ClimateVariable")]
#[serde(rename_all = "camelCase")]
pub enum ClimateVariable {
    HeatDays,
    IceDays,
    TropicalNights,
    FrostDays,
    DryDays,
    HeavyRainDays,
}

impl ClimateVariable {
    pub const ALL: &'static [Self] = &[
        Self::HeatDays,
        Self::IceDays,
        Self::TropicalNights,
        Self::FrostDays,
        Self::DryDays,
        Self::HeavyRainDays,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::HeatDays => "heatDays",
            Self::IceDays => "iceDays",
            Self::TropicalNights => "tropicalNights",
            Self::FrostDays => "frostDays",
            Self::DryDays => "dryDays",
            Self::HeavyRainDays => "heavyRainDays",
        }
    }

    /// Returns the dataset suffix and `GeoEngine` expression for this variable.
    ///
    /// Expressions operate on daily values `A`: temperature in Kelvin (converted via
    /// `A - 273.15` to °C) or precipitation in mm/s (converted via `A * 86400` to mm/day).
    /// Thresholds follow WMO climate-index definitions:
    /// heat ≥ 30 °C, ice/frost < 0 °C, tropical night > 20 °C, dry < 1 mm, heavy rain > 20 mm.
    pub fn properties(self) -> ClimateVariableProperties {
        match self {
            ClimateVariable::HeatDays => ClimateVariableProperties {
                name: "Heat Days",
                dataset_variable_suffix: "tasmax",
                expression: "let c = (A - 273.15); if c >= 30 { 1 } else { 0 }",
            },
            ClimateVariable::IceDays => ClimateVariableProperties {
                name: "Ice Days",
                dataset_variable_suffix: "tasmax",
                expression: "let c = (A - 273.15); if c < 0 { 1 } else { 0 }",
            },
            ClimateVariable::TropicalNights => ClimateVariableProperties {
                name: "Tropical Nights",
                dataset_variable_suffix: "tasmin",
                expression: "let c = (A - 273.15); if c > 20 { 1 } else { 0 }",
            },
            ClimateVariable::FrostDays => ClimateVariableProperties {
                name: "Frost Days",
                dataset_variable_suffix: "tasmin",
                expression: "let c = (A - 273.15); if c < 0 { 1 } else { 0 }",
            },
            ClimateVariable::DryDays => ClimateVariableProperties {
                name: "Dry Days",
                dataset_variable_suffix: "pr",
                expression: "if (A * 86400) < 1 { 1 } else { 0 }",
            },
            ClimateVariable::HeavyRainDays => ClimateVariableProperties {
                name: "Heavy Rain Days",
                dataset_variable_suffix: "pr",
                expression: "if (A * 86400) > 20 { 1 } else { 0 }",
            },
        }
    }
}

/// A climate variable selected for computation.
#[derive(Debug, Clone)]
pub struct ClimateVariableRequest {
    pub variable: ClimateVariable,
}

impl ClimateVariableRequest {
    pub fn new(variable: ClimateVariable) -> Self {
        Self { variable }
    }
}

/// NEX-GDDP-CMIP6 climate model (loaded from registry).
#[derive(Debug, Clone)]
pub struct ClimateModelProperties {
    pub id: String,
    pub variant: String,
    pub grid: String,
    pub scenarios: Vec<ClimateScenario>,
}

/// Shared Socioeconomic Pathway scenario.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone, PartialEq, Eq, Hash)]
#[schema(title = "ClimateScenario")]
#[serde(rename_all = "lowercase")]
pub enum ClimateScenario {
    Historical,
    Ssp245,
    Ssp585,
}

impl ClimateScenario {
    pub const ALL: &'static [Self] = &[Self::Historical, Self::Ssp245, Self::Ssp585];

    pub const FUTURE: &'static [Self] = &[Self::Ssp245, Self::Ssp585];

    pub fn name(self) -> &'static str {
        match self {
            Self::Historical => "historical",
            Self::Ssp245 => "ssp245",
            Self::Ssp585 => "ssp585",
        }
    }
    pub fn properties(self) -> ClimateScenarioProperties {
        match self {
            ClimateScenario::Historical => ClimateScenarioProperties {
                name: "Historical (1950–2014)",
                dataset_prefix: "historical",
                scenario: ClimateScenario::Historical,
            },
            ClimateScenario::Ssp245 => ClimateScenarioProperties {
                name: "SSP2-4.5 (Intermediate emissions)",
                dataset_prefix: "ssp245",
                scenario: ClimateScenario::Ssp245,
            },
            ClimateScenario::Ssp585 => ClimateScenarioProperties {
                name: "SSP5-8.5 (High emissions)",
                dataset_prefix: "ssp585",
                scenario: ClimateScenario::Ssp585,
            },
        }
    }
}

/// Resolved properties for a climate scenario (dataset prefix, display name).
#[derive(Debug, Clone)]
pub struct ClimateScenarioProperties {
    pub name: &'static str,
    pub dataset_prefix: &'static str,
    pub scenario: ClimateScenario,
}

pub(crate) fn default_year_begin() -> Year {
    Year(2050)
}

/// First year with available NEX-GDDP-CMIP6 future data.
pub(crate) const FUTURE_START_YEAR: u16 = 2015;
/// Last year with available NEX-GDDP-CMIP6 future data.
pub(crate) const FUTURE_END_YEAR: u16 = 2100;
/// First year with available NEX-GDDP-CMIP6 historical data.
pub(crate) const HISTORICAL_START_YEAR: u16 = 1950;
/// Last year with available NEX-GDDP-CMIP6 historical data.
pub(crate) const HISTORICAL_END_YEAR: u16 = 2014;
/// Days in a Julian year (IAU: 365 d + 1 leap day / 4). Used to convert annual occurrence counts to probabilities.
pub(crate) const DAYS_PER_JULIAN_YEAR: f64 = 365.25;

pub(crate) fn default_reference_year_begin() -> Year {
    Year(1981)
}

fn default_year_range() -> YearRange {
    YearRange(30)
}

fn default_variables() -> Vec<ClimateVariable> {
    ClimateVariable::ALL.to_vec()
}

// ponytail: default models loaded from registry at startup; empty here, populated in mod.rs
fn default_models() -> Vec<String> {
    Vec::new()
}

/// User-supplied inputs for the climate risk process.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClimateRiskInputs {
    pub coordinate: PointGeoJsonInput,
    #[serde(default = "default_year_begin")]
    #[schema(minimum = 2015, maximum = 2100)]
    pub year_begin: Year,
    #[serde(default = "default_year_range")]
    #[schemars(default = "default_year_range")]
    pub year_range: YearRange,
    #[schemars(default = "default_reference_year_begin")]
    #[schema(minimum = 1950, maximum = 2014)]
    pub reference_year_begin: Year,
    #[serde(default = "default_variables")]
    pub variables: Vec<ClimateVariable>,
    #[serde(default = "default_models")]
    pub models: Vec<String>,
}

/// Aggregated statistics for a single climate variable across models.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Clone)]
pub struct ClimateVariableResult {
    pub max: Days,
    pub min: Days,
    pub mean: Days,
    pub median: Days,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_probability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_members: Option<HashMap<String, f64>>,
}

/// A single row in the climate risk `DataResource` output.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClimateRiskRow {
    pub variable: String,
    pub scenario: String,
    /// Mean number of days per year in the analysis period.
    pub mean: Days,
    /// Median number of days per year in the analysis period.
    pub median: Days,
    /// Minimum number of days per year across models.
    pub min: Days,
    /// Maximum number of days per year across models.
    pub max: Days,
    /// Occurrence probability as a ratio from 0 to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_probability: Option<f64>,
    /// Difference in days per year from the reference period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly: Option<Days>,
    /// Ready-to-display occurrence-probability label, e.g. "7 · high (3 %)".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_probability_label: Option<String>,
    /// Cell color for the occurrence probability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_probability_color: Option<String>,
    /// Ready-to-display anomaly label, e.g. "+10 days (+20 %)".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_label: Option<String>,
    /// Cell color for the anomaly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly_color: Option<String>,
}

/// A single row in the raw ensemble data output.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClimateRiskRawRow {
    pub variable: String,
    pub scenario: String,
    pub model: String,
    pub value: f64,
}

/// Output of the climate risk process: summary table and raw ensemble data.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClimateRiskOutputs {
    pub inputs: Option<ClimateRiskInputs>,
    /// Analysis window as `"2041–2070"`, used for display in result headlines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis_period: Option<String>,
    /// Reference window used for anomalies as `"2006–2025"`, `None` when no reference period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_period: Option<String>,
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub climate_risk: Option<DataResource<Vec<ClimateRiskRow>>>,
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub raw_ensemble_data: Option<DataResource<Vec<ClimateRiskRawRow>>>,
}

/// Column title for the anomaly: "Anomaly (days/year compared to 2006–2025)".
/// The analysis period lives in the resource name, so it is not repeated here.
pub(crate) fn anomaly_title(reference_period: Option<&str>) -> String {
    match reference_period {
        Some(period) => format!("Anomaly (days/year compared to {period})"),
        None => "Anomaly (days/year)".to_string(),
    }
}

/// Existing FMEA/ISO-inspired occurrence-probability classes, ordered from
/// rarest to most likely. The thresholds and presentation values are kept
/// together so labels and colors cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProbabilityClass {
    ExtremelyLow,
    VeryLow,
    Low,
    ModeratelyLow,
    Moderate,
    ModeratelyHigh,
    High,
    VeryHigh,
    ExtremelyHigh,
    Extreme,
}

impl ProbabilityClass {
    const ALL: [Self; 10] = [
        Self::ExtremelyLow,
        Self::VeryLow,
        Self::Low,
        Self::ModeratelyLow,
        Self::Moderate,
        Self::ModeratelyHigh,
        Self::High,
        Self::VeryHigh,
        Self::ExtremelyHigh,
        Self::Extreme,
    ];

    pub(crate) fn from_probability(probability: f64) -> Self {
        Self::ALL
            .iter()
            .rev()
            .find(|class| probability >= class.threshold())
            .copied()
            .unwrap_or(Self::ExtremelyLow)
    }

    pub(crate) fn number(self) -> u8 {
        match self {
            Self::ExtremelyLow => 1,
            Self::VeryLow => 2,
            Self::Low => 3,
            Self::ModeratelyLow => 4,
            Self::Moderate => 5,
            Self::ModeratelyHigh => 6,
            Self::High => 7,
            Self::VeryHigh => 8,
            Self::ExtremelyHigh => 9,
            Self::Extreme => 10,
        }
    }

    pub(crate) fn threshold(self) -> f64 {
        1.0 / f64::from(self.return_period_years())
    }

    /// Return period represented by this existing FMEA/ISO-inspired class.
    pub(crate) fn return_period_years(self) -> u32 {
        match self {
            Self::ExtremelyLow => 20_000,
            Self::VeryLow => 5_000,
            Self::Low => 1_000,
            Self::ModeratelyLow => 500,
            Self::Moderate => 250,
            Self::ModeratelyHigh => 100,
            Self::High => 50,
            Self::VeryHigh => 20,
            Self::ExtremelyHigh => 10,
            Self::Extreme => 5,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::ExtremelyLow => "extremely low",
            Self::VeryLow => "very low",
            Self::Low => "low",
            Self::ModeratelyLow => "moderately low",
            Self::Moderate => "moderate",
            Self::ModeratelyHigh => "moderately high",
            Self::High => "high",
            Self::VeryHigh => "very high",
            Self::ExtremelyHigh => "extremely high",
            Self::Extreme => "extreme",
        }
    }

    fn color(self) -> &'static str {
        match self {
            Self::ExtremelyLow => "#66bb6a",
            Self::VeryLow => "#2e7d32",
            Self::Low => "#fdd835",
            Self::ModeratelyLow => "#f9a825",
            Self::Moderate => "#fb8c00",
            Self::ModeratelyHigh => "#ef6c00",
            Self::High => "#e53935",
            Self::VeryHigh => "#c62828",
            Self::ExtremelyHigh => "#8e24aa",
            Self::Extreme => "#4a148c",
        }
    }
}

/// Ready-to-display occurrence-probability label, e.g. "7 · high (3 %)".
pub(crate) fn probability_label(p: f64) -> String {
    let class = ProbabilityClass::from_probability(p);
    let pct = format!("{:.1}", p * 100.0)
        .trim_end_matches(".0")
        .to_string();
    format!("{} · {} ({pct} %)", class.number(), class.label())
}

pub(crate) fn probability_color(p: f64) -> String {
    ProbabilityClass::from_probability(p).color().to_string()
}

/// Shared 7-stop divergent gradient for the anomaly color scale, ordered from -100 % to +100 %.
const ANOMALY_PALETTE: [&str; 7] = [
    "#2166ac", "#67a9cf", "#d1e5f0", "#f7f7f7", "#fddbc7", "#ef8a62", "#b2182b",
];

/// Quantizes a percentage change (-100..=+100) to the nearest of the 7 shared class stops.
/// Values outside the range clamp to the extremes; negative = blue, zero = white, positive = red.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "The rounded palette index is bounded to -3..=3 before casting, so no truncation or sign loss can occur."
)]
pub(crate) fn percentage_color(pct: f64) -> String {
    let t = ((3.0 * pct / 100.0).round() as i32).clamp(-3, 3);
    ANOMALY_PALETTE[(t + 3) as usize].to_string()
}

/// Percentage change of the analysis mean relative to the reference mean, unclamped.
/// A missing/zero reference mean maps to the extreme percentages by sign of the anomaly.
pub(crate) fn anomaly_pct(analysis_mean: f64, reference_mean: f64) -> f64 {
    if (analysis_mean - reference_mean).abs() <= f64::EPSILON {
        0.0
    } else if reference_mean.abs() <= f64::EPSILON {
        (analysis_mean - reference_mean).signum() * 100.0
    } else {
        (analysis_mean - reference_mean) / reference_mean * 100.0
    }
}

/// Formats a signed value with `decimals` places, trailing zeros trimmed, e.g. 10.0 -> "+10".
fn format_signed(value: f64, decimals: usize) -> String {
    let sign = if value > 0.0 {
        "+"
    } else if value < 0.0 {
        "-"
    } else {
        ""
    };
    let digits = format!("{:.decimals$}", value.abs(), decimals = decimals);
    let trimmed = digits.trim_end_matches('0').trim_end_matches('.');
    format!("{sign}{trimmed}")
}

/// Ready-to-display anomaly label, e.g. "+10 days (+20 %)".
pub(crate) fn anomaly_label(anomaly_days: f64, pct: f64) -> String {
    format!(
        "{} days ({} %)",
        format_signed(anomaly_days, 2),
        format_signed(pct, 1)
    )
}

/// Dataset suffix and `GeoEngine` expression for a climate variable.
pub struct ClimateVariableProperties {
    pub(crate) name: &'static str,
    pub(crate) dataset_variable_suffix: &'static str,
    pub(crate) expression: &'static str,
}

impl ClimateVariableProperties {
    pub fn name_string(&self) -> String {
        self.name.to_string()
    }
    pub fn measurement_string() -> String {
        "Days".to_string()
    }
    pub fn measurement_unit() -> String {
        "Days".to_string()
    }
    pub fn expression_string(&self) -> String {
        self.expression.to_string()
    }
    pub fn expression_dtype() -> RasterDataType {
        RasterDataType::I8
    }
    pub fn year_agg_dtype() -> RasterDataType {
        RasterDataType::U16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn it_exposes_climate_variable_properties() {
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
    fn it_exposes_climate_scenario_properties() {
        for (scenario, expected_name, expected_prefix) in [
            (
                ClimateScenario::Historical,
                "Historical (1950–2014)",
                "historical",
            ),
            (
                ClimateScenario::Ssp245,
                "SSP2-4.5 (Intermediate emissions)",
                "ssp245",
            ),
            (
                ClimateScenario::Ssp585,
                "SSP5-8.5 (High emissions)",
                "ssp585",
            ),
        ] {
            let props = scenario.properties();
            assert_eq!(props.scenario, scenario);
            assert_eq!(props.name, expected_name);
            assert_eq!(props.dataset_prefix, expected_prefix);
        }
    }

    #[test]
    fn it_provides_climate_variable_property_accessors() {
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
    fn it_exposes_scenario_names() {
        for scenario in ClimateScenario::ALL {
            assert_eq!(
                scenario.properties().name,
                match scenario {
                    ClimateScenario::Historical => "Historical (1950–2014)",
                    ClimateScenario::Ssp245 => "SSP2-4.5 (Intermediate emissions)",
                    ClimateScenario::Ssp585 => "SSP5-8.5 (High emissions)",
                }
            );
        }
    }

    #[test]
    fn it_maps_every_probability_class_number_label_and_color() {
        for class in ProbabilityClass::ALL {
            let number = class.number();
            assert!((1..=10).contains(&number));
            assert!(!class.label().is_empty());
            assert!(class.color().starts_with('#'));
            assert!(class.return_period_years() > 0);
            assert!(class.threshold() > 0.0);
        }
        assert_eq!(ProbabilityClass::ModeratelyLow.number(), 4);
        assert_eq!(ProbabilityClass::ModeratelyLow.label(), "moderately low");
        assert_eq!(ProbabilityClass::VeryLow.color(), "#2e7d32");
        assert_eq!(ProbabilityClass::ExtremelyHigh.color(), "#8e24aa");
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
            mean: Days(50.0),
            median: Days(50.0),
            min: Days(0.0),
            max: Days(100.0),
            occurrence_probability: Some(0.03),
            anomaly: Some(Days(10.0)),
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
}
