use crate::processes::parameters::{
    BoundingBox, DataResource, DataResourceSchema, PointGeoJsonInput, Year, YearRange,
    nearest_containing,
};
use geoengine_api_client::models::RasterDataType;
use geojson::PointType;
use ogcapi::types::common::Crs;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone, PartialEq, Eq, Hash)]
#[schema(title = "CordexRegion")]
pub enum CordexRegion {
    Eur,
}

impl CordexRegion {
    pub const ALL: &'static [Self] = &[Self::Eur];

    pub fn name(self) -> &'static str {
        match self {
            Self::Eur => "Eur",
        }
    }
    pub fn properties(self) -> CordexRegionProperties {
        match self {
            CordexRegion::Eur => CordexRegionProperties {
                name: "Europe",
                dataset_prefix: "EUR11",
                bounding_box: BoundingBox::new(-10.0, 34.0, 30.0, 72.0, Crs::from_epsg(4326)),
                region: CordexRegion::Eur,
            },
        }
    }

    pub fn point_to_region(point: &PointType) -> Option<Self> {
        nearest_containing(
            point,
            Self::ALL
                .iter()
                .map(|region| (*region, region.properties().bounding_box)),
        )
    }
}

#[derive(Debug)]
pub struct CordexRegionProperties {
    pub name: &'static str,
    pub dataset_prefix: &'static str,
    pub bounding_box: BoundingBox,
    pub region: CordexRegion,
}

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

#[derive(Debug, Clone)]
pub struct ClimateVariableRequest {
    pub variable: ClimateVariable,
}

impl ClimateVariableRequest {
    pub fn new(variable: ClimateVariable) -> Self {
        Self { variable }
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Clone, PartialEq, Eq, Copy, Hash)]
#[schema(title = "ClimateModel")]
pub enum CordexModel {
    #[serde(rename = "MPI-M-MPI-ESM-LR")]
    MpiMmpiEsmLr,
    #[serde(rename = "MOHC-HadGEM2-ES")]
    MohcHadgem2Es,
}

impl CordexModel {
    pub const ALL: &'static [Self] = &[Self::MpiMmpiEsmLr, Self::MohcHadgem2Es];

    pub fn name(self) -> &'static str {
        match self {
            Self::MpiMmpiEsmLr => "MPI-M-MPI-ESM-LR",
            Self::MohcHadgem2Es => "MOHC-HadGEM2-ES",
        }
    }
    pub fn properties(self) -> CordexModelProperties {
        match self {
            CordexModel::MpiMmpiEsmLr => CordexModelProperties {
                name: "MPI-M-MPI-ESM-LR",
                dataset_prefix: "MPI-M-MPI-ESM-LR",
                region: CordexRegion::Eur,
                scenarios: vec![
                    ClimateScenario::Rcp26,
                    ClimateScenario::Rcp45,
                    ClimateScenario::Rcp85,
                ],
                model: CordexModel::MpiMmpiEsmLr,
            },
            CordexModel::MohcHadgem2Es => CordexModelProperties {
                name: "MOHC-HadGEM2-ES",
                dataset_prefix: "MOHC-HadGEM2-ES",
                region: CordexRegion::Eur,
                scenarios: vec![
                    ClimateScenario::Rcp26,
                    ClimateScenario::Rcp45,
                    ClimateScenario::Rcp85,
                ],
                model: CordexModel::MohcHadgem2Es,
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CordexModelProperties {
    pub name: &'static str,
    pub dataset_prefix: &'static str,
    pub region: CordexRegion,
    pub scenarios: Vec<ClimateScenario>,
    pub model: CordexModel,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone, PartialEq, Eq, Hash)]
#[schema(title = "ClimateScenario")]
#[serde(rename_all = "lowercase")]
pub enum ClimateScenario {
    Rcp26,
    Rcp45,
    Rcp85,
}

impl ClimateScenario {
    pub const ALL: &'static [Self] = &[Self::Rcp26, Self::Rcp45, Self::Rcp85];

    pub fn name(self) -> &'static str {
        match self {
            Self::Rcp26 => "rcp26",
            Self::Rcp45 => "rcp45",
            Self::Rcp85 => "rcp85",
        }
    }
    pub fn properties(self) -> ClimateScenarioProperties {
        match self {
            ClimateScenario::Rcp26 => ClimateScenarioProperties {
                name: "RCP 2.6 (Low emissions)",
                dataset_prefix: "rcp26",
                scenario: ClimateScenario::Rcp26,
            },
            ClimateScenario::Rcp45 => ClimateScenarioProperties {
                name: "RCP 4.5 (Intermediate emissions)",
                dataset_prefix: "rcp45",
                scenario: ClimateScenario::Rcp45,
            },
            ClimateScenario::Rcp85 => ClimateScenarioProperties {
                name: "RCP 8.5 (High emissions)",
                dataset_prefix: "rcp85",
                scenario: ClimateScenario::Rcp85,
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClimateScenarioProperties {
    pub name: &'static str,
    pub dataset_prefix: &'static str,
    pub scenario: ClimateScenario,
}

fn default_year_begin() -> Year {
    Year(2014)
}

pub(crate) const DATA_START_YEAR: u16 = 2006;
// Climate values are aggregated using a Julian year.
pub(crate) const DAYS_PER_JULIAN_YEAR: f64 = 365.25;

#[allow(clippy::unnecessary_wraps)]
fn default_reference_year_begin() -> Option<Year> {
    Some(Year(DATA_START_YEAR))
}

fn default_year_range() -> YearRange {
    YearRange(20)
}

fn default_variables() -> Vec<ClimateVariable> {
    ClimateVariable::ALL.to_vec()
}

fn default_models() -> Vec<CordexModel> {
    CordexModel::ALL.to_vec()
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClimateRiskInputs {
    pub coordinate: PointGeoJsonInput,
    #[serde(default = "default_year_begin")]
    #[schema(minimum = 2006, maximum = 2100)]
    pub year_begin: Year,
    #[serde(default = "default_year_range")]
    #[schemars(default = "default_year_range")]
    pub year_range: YearRange,
    #[serde(default)]
    #[schemars(default = "default_reference_year_begin")]
    pub reference_year_begin: Option<Year>,
    #[serde(default = "default_variables")]
    pub variables: Vec<ClimateVariable>,
    #[serde(default = "default_models")]
    pub models: Vec<CordexModel>,
    #[serde(default)]
    pub region: Option<CordexRegion>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Clone)]
pub struct ClimateVariableResult {
    pub max: f64,
    pub min: f64,
    pub mean: f64,
    pub median: f64,
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
    pub mean: f64,
    /// Median number of days per year in the analysis period.
    pub median: f64,
    /// Minimum number of days per year across models.
    pub min: f64,
    /// Maximum number of days per year across models.
    pub max: f64,
    /// Occurrence probability as a ratio from 0 to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_probability: Option<f64>,
    /// Difference in days per year from the reference period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anomaly: Option<f64>,
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
