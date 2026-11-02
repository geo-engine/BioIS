use crate::processes::parameters::{
    DataResource, DataResourceSchema, Days, PointGeoJsonInput, Year, YearRange,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use utoipa::ToSchema;

/// Climate variable to compute (WMO-based daily threshold indicators).
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone, PartialEq, Eq, Hash)]
#[schema(title = "ClimateVariable")]
#[schemars(rename = "ClimateVariable")]
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

    /// The display name of the indicator, e.g. "Heat Days".
    ///
    /// The counts are precomputed by `notebooks/nexgddp_cmip6_aggregate_yearly.ipynb` from the
    /// daily data with WMO climate-index thresholds: heat ≥ 30 °C (tasmax), ice < 0 °C (tasmax),
    /// tropical night > 20 °C (tasmin), frost < 0 °C (tasmin), dry < 1 mm (pr),
    /// heavy rain > 20 mm (pr).
    pub fn display_name(self) -> &'static str {
        match self {
            Self::HeatDays => "Heat Days",
            Self::IceDays => "Ice Days",
            Self::TropicalNights => "Tropical Nights",
            Self::FrostDays => "Frost Days",
            Self::DryDays => "Dry Days",
            Self::HeavyRainDays => "Heavy Rain Days",
        }
    }
}

/// NEX-GDDP-CMIP6 climate model (loaded from registry).
#[derive(Debug, Clone)]
pub struct ClimateModelProperties {
    pub id: String,
    // variant + grid identify the CMIP6 realization; consumed by the ingestion notebook when
    // building kerchunk reference paths, not by the backend query path.
    #[allow(
        dead_code,
        reason = "CMIP6 realization identity, used for data ingestion only"
    )]
    pub variant: String,
    #[allow(dead_code, reason = "CMIP6 grid label, used for data ingestion only")]
    pub grid: String,
    pub scenarios: Vec<ClimateScenario>,
}

/// Shared Socioeconomic Pathway scenario.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    JsonSchema,
    ToSchema,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
)]
#[schema(title = "ClimateScenario")]
#[schemars(rename = "ClimateScenario")]
#[serde(rename_all = "lowercase")]
pub enum ClimateScenario {
    #[default]
    Historical,
    Ssp245,
    Ssp370,
    Ssp585,
}

impl ClimateScenario {
    pub const ALL: &'static [Self] = &[Self::Historical, Self::Ssp245, Self::Ssp370, Self::Ssp585];

    pub const FUTURE: &'static [Self] = &[Self::Ssp245, Self::Ssp370, Self::Ssp585];

    pub fn name(self) -> &'static str {
        match self {
            Self::Historical => "historical",
            Self::Ssp245 => "ssp245",
            Self::Ssp370 => "ssp370",
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
            ClimateScenario::Ssp370 => ClimateScenarioProperties {
                name: "SSP3-7.0 (Moderate-high emissions)",
                dataset_prefix: "ssp370",
                scenario: ClimateScenario::Ssp370,
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
/// Shortest and longest aggregation window in years, cf. [`YearRange`].
pub(crate) const MIN_YEAR_RANGE: u16 = 5;
pub(crate) const MAX_YEAR_RANGE: u16 = 30;
/// Length of the reference period: a WMO climate normal, independent of the analysis window, so
/// that the reference 10-year level rests on enough years even for short analysis windows.
pub(crate) const REFERENCE_YEARS: u16 = 30;

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
    #[schema(minimum = 2015, maximum = 2096)]
    pub year_begin: Year,
    #[serde(default = "default_year_range")]
    #[schemars(default = "default_year_range")]
    pub year_range: YearRange,
    #[schemars(default = "default_reference_year_begin")]
    #[schema(minimum = 1950, maximum = 2010)]
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
    pub raw_members: Option<HashMap<String, f64>>,
    /// Standard error of `mean` from the year-to-year variability within the window.
    #[serde(skip)]
    pub standard_error: Option<f64>,
}

/// One model's yearly values for a variable/scenario over the queried window.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ModelYears {
    /// Mean number of event days per year.
    pub mean: f64,
    /// Number of event days of each year in the window.
    pub values: Vec<f64>,
    /// Variance of the yearly values around their linear trend; `None` for fewer than 3 years.
    pub residual_variance: Option<f64>,
}

/// A single row in the climate risk `DataResource` output.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClimateRiskRow {
    pub variable: String,
    pub scenario: String,
    /// The scenario as the output id it is reported under; the display name is `scenario`.
    #[serde(skip)]
    pub scenario_id: ClimateScenario,
    /// Mean number of days per year in the analysis period.
    pub mean: Days,
    /// Median number of days per year in the analysis period.
    pub median: Days,
    /// Minimum number of days per year across models.
    pub min: Days,
    /// Maximum number of days per year across models.
    pub max: Days,
    /// Annual probability of a 10-year event of the reference period, as a ratio from 0 to 1:
    /// the share of years (across models) with more days than the model's reference 1-in-10-year
    /// level. See `ten_year_event_probabilities`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_probability: Option<f64>,
    /// How many times more often the 10-year event occurs than in the reference period
    /// (IPCC "probability ratio"); `None` if it never occurred there.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_ratio: Option<f64>,
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
    /// Half width of the 95 % confidence interval of `mean` from the year-to-year variability
    /// within the window (around its trend), in days per year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<Days>,
    /// Ready-to-display mean with its uncertainty, e.g. "14.1 ± 2.9".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_label: Option<String>,
    /// Standard error of `mean`, for comparing scenarios.
    #[serde(skip)]
    pub standard_error: Option<f64>,
    /// Multi-year mean per model, for the model agreement between scenarios.
    #[serde(skip)]
    pub model_means: HashMap<String, f64>,
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
    /// Reference window used for anomalies as `"1981–2010"`, `None` when no reference period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_period: Option<String>,
    /// Summary rows of all computed scenarios, reported as one table per scenario.
    #[serde(default)]
    pub climate_risk: Vec<ClimateRiskRow>,
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub raw_ensemble_data: Option<DataResource<Vec<ClimateRiskRawRow>>>,
    /// Per scenario, the variables that cannot be told apart from other scenarios in this window.
    #[serde(skip)]
    pub scenario_notes: BTreeMap<ClimateScenario, String>,
}

/// Column title for the anomaly: "Anomaly (days/year compared to 1981–2010)".
/// The analysis period lives in the resource name, so it is not repeated here.
pub(crate) fn anomaly_title(reference_period: Option<&str>) -> String {
    match reference_period {
        Some(period) => format!("Anomaly (days/year compared to {period})"),
        None => "Anomaly (days/year)".to_string(),
    }
}

/// Column title for the occurrence probability: "Occurrence Probability (10-year event of 1981–2010)".
pub(crate) fn occurrence_probability_title(reference_period: Option<&str>) -> String {
    match reference_period {
        Some(period) => format!("Occurrence Probability (10-year event of {period})"),
        None => "Occurrence Probability (10-year event)".to_string(),
    }
}

/// Column title for the risk ratio: "Risk Ratio (vs. 1981–2010)".
pub(crate) fn risk_ratio_title(reference_period: Option<&str>) -> String {
    match reference_period {
        Some(period) => format!("Risk Ratio (vs. {period})"),
        None => "Risk Ratio".to_string(),
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

/// Occurrence-probability label with the classes of the 95 % interval `low..=high`: like
/// [`probability_label`] if they are one class, otherwise e.g. "7–10 · high to extreme (12 %)".
pub(crate) fn probability_range_label(p: f64, low: f64, high: f64) -> String {
    let (low, high) = (
        ProbabilityClass::from_probability(low),
        ProbabilityClass::from_probability(high),
    );
    if low == high {
        return probability_label(p);
    }
    let pct = format!("{:.1}", p * 100.0)
        .trim_end_matches(".0")
        .to_string();
    format!(
        "{}–{} · {} to {} ({pct} %)",
        low.number(),
        high.number(),
        low.label(),
        high.label()
    )
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
/// `None` for a zero reference mean with a change, where no percentage exists.
pub(crate) fn anomaly_pct(analysis_mean: f64, reference_mean: f64) -> Option<f64> {
    if (analysis_mean - reference_mean).abs() <= f64::EPSILON {
        Some(0.0)
    } else if reference_mean.abs() <= f64::EPSILON {
        None
    } else {
        Some((analysis_mean - reference_mean) / reference_mean * 100.0)
    }
}

/// Anomaly color; without a percentage (zero reference), the strongest stop by sign.
pub(crate) fn anomaly_color(anomaly_days: f64, pct: Option<f64>) -> String {
    percentage_color(pct.unwrap_or(anomaly_days.signum() * 100.0))
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

/// Ready-to-display anomaly label, e.g. "+10 days (+20 %)", or "+2 days (new)" when the event
/// did not occur in the reference period.
pub(crate) fn anomaly_label(anomaly_days: f64, pct: Option<f64>) -> String {
    let days = format_signed(anomaly_days, 2);
    match pct {
        Some(pct) => format!("{days} days ({} %)", format_signed(pct, 1)),
        None => format!("{days} days (new)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn it_names_climate_variables() {
        assert_eq!(ClimateVariable::HeatDays.display_name(), "Heat Days");
        assert_eq!(
            ClimateVariable::HeavyRainDays.display_name(),
            "Heavy Rain Days"
        );
        assert_eq!(ClimateVariable::HeavyRainDays.name(), "heavyRainDays");
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
                ClimateScenario::Ssp370,
                "SSP3-7.0 (Moderate-high emissions)",
                "ssp370",
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
    fn it_exposes_scenario_names() {
        for scenario in ClimateScenario::ALL {
            assert_eq!(
                scenario.properties().name,
                match scenario {
                    ClimateScenario::Historical => "Historical (1950–2014)",
                    ClimateScenario::Ssp245 => "SSP2-4.5 (Intermediate emissions)",
                    ClimateScenario::Ssp370 => "SSP3-7.0 (Moderate-high emissions)",
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
        assert_eq!(
            probability_range_label(0.4, 0.23, 0.59),
            "10 · extreme (40 %)"
        );
        assert_eq!(
            probability_range_label(0.12, 0.04, 0.3),
            "7–10 · high to extreme (12 %)"
        );
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
        assert_abs_diff_eq!(anomaly_pct(120.0, 100.0).unwrap(), 20.0);
        assert_abs_diff_eq!(anomaly_pct(50.0, 100.0).unwrap(), -50.0);
        assert_abs_diff_eq!(anomaly_pct(0.0, 0.0).unwrap(), 0.0);
        // No percentage of a zero reference: the event is new.
        assert_eq!(anomaly_pct(10.0, 0.0), None);
    }

    #[test]
    fn it_anomaly_label_maps_days_and_raw_pct() {
        assert_eq!(anomaly_label(10.0, Some(20.0)), "+10 days (+20 %)");
        assert_eq!(anomaly_label(-5.0, Some(-10.0)), "-5 days (-10 %)");
        assert_eq!(anomaly_label(0.0, Some(0.0)), "0 days (0 %)");
        assert_eq!(anomaly_label(10.5, Some(33.3)), "+10.5 days (+33.3 %)");
        assert_eq!(
            anomaly_label(10.0, Some(250.0)),
            "+10 days (+250 %)",
            "label shows the raw percentage, the color clamps separately"
        );
        assert_eq!(anomaly_label(2.0, None), "+2 days (new)");
    }

    #[test]
    fn it_anomaly_color_falls_back_to_the_strongest_stop_without_pct() {
        assert_eq!(anomaly_color(2.0, None), "#b2182b");
        assert_eq!(anomaly_color(2.0, Some(0.0)), "#f7f7f7");
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
            scenario: "ssp245".to_string(),
            scenario_id: ClimateScenario::Ssp245,
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
            ..Default::default()
        };
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["occurrenceProbabilityLabel"], "7 · high (3 %)");
        assert_eq!(json["occurrenceProbabilityColor"], "#e53935");
        assert_eq!(json["anomalyLabel"], "+10 days (+20 %)");
        assert_eq!(json["anomalyColor"], "#fddbc7");
        assert!(json.get("scenarioId").is_none());
    }
}
