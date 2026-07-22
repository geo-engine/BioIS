use crate::processes::parameters::{
    Area, DataResource, Fields, SquareMeter, TableSchemaField, TableSchemaType, UnitForArea,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy)]
pub struct LandUseSummaryRow {
    /// Land-use summary type
    pub land_use_type: LandUseSummaryRowType,
    /// Previous year (if available)
    pub previous_year: Option<SquareMeter>,
    /// Reporting year
    pub reporting_year: SquareMeter,
    /// % change
    pub percentage_change: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
#[allow(
    clippy::enum_variant_names,
    reason = "These names are used in the VSME standard and should be kept as-is."
)]
pub enum LandUseSummaryRowType {
    TotalSealedArea,
    TotalNatureOnSiteArea,
    TotalNatureOffSiteArea,
    TotalUseOfLand,
}

impl LandUseSummaryRowType {
    fn to_display_str(self) -> &'static str {
        match self {
            LandUseSummaryRowType::TotalSealedArea => "Total sealed area",
            LandUseSummaryRowType::TotalNatureOnSiteArea => "Total nature-oriented area on-site",
            LandUseSummaryRowType::TotalNatureOffSiteArea => "Total nature-oriented area off-site",
            LandUseSummaryRowType::TotalUseOfLand => "Total use of land",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LandUseSummaryRowOutput {
    /// Land-use type based on [`LandUseSummaryRowType`]
    pub land_use_type: &'static str,
    /// Previous year (if available)
    pub previous_year: Option<Area>,
    /// Reporting year
    pub reporting_year: Area,
    /// % change
    pub percentage_change: Option<f64>,
}

fn land_use_summary_row_to_output(
    row: LandUseSummaryRow,
    unit_for_area: UnitForArea,
) -> LandUseSummaryRowOutput {
    LandUseSummaryRowOutput {
        land_use_type: row.land_use_type.to_display_str(),
        previous_year: row
            .previous_year
            .map(|value| Area::from_square_meters(value, unit_for_area)),
        reporting_year: Area::from_square_meters(row.reporting_year, unit_for_area),
        percentage_change: row.percentage_change,
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(
    clippy::struct_field_names,
    reason = "These names are used in the VSME standard and should be kept as-is."
)]
pub struct LandUseSummary {
    /// Total sealed area
    pub total_sealed_area: LandUseSummaryRow,
    /// Total nature-oriented area on-site
    pub total_nature_on_site_area: LandUseSummaryRow,
    /// Total nature-oriented area off-site
    pub total_nature_off_site_area: LandUseSummaryRow,
    /// Total use of land
    pub total_use_of_land: LandUseSummaryRow,
}

/// If the previous year data is available, it will be used to calculate the percentage change for each land use category.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_field_names,
    reason = "These names are used in the VSME standard and should be kept as-is."
)]
pub struct PreviousLandUseSummary {
    /// Total sealed area
    #[schemars(title = "Total sealed area")]
    pub total_sealed_area: f64,
    /// Total nature-oriented area on-site
    #[schemars(title = "Total nature-oriented area on-site")]
    pub total_nature_on_site_area: f64,
    /// Total nature-oriented area off-site
    #[schemars(title = "Total nature-oriented area off-site")]
    pub total_nature_off_site_area: f64,
    /// Total use of land
    #[schemars(title = "Total use of land")]
    pub total_use_of_land: f64,
    /// Unit for area values (e.g., "ha" for hectares, "m²" for square meters)
    pub unit_for_area: UnitForArea,
}

/// Typed version of `PreviousLandUseSummary` where the area values are represented as `Area` types instead of raw `f64` values.
#[derive(Debug, Clone, Copy)]
#[allow(
    clippy::struct_field_names,
    reason = "These names are used in the VSME standard and should be kept as-is."
)]
pub struct TypedPreviousLandUseSummary {
    /// Total sealed area
    pub total_sealed_area: Option<Area>,
    /// Total nature-oriented area on-site
    pub total_nature_on_site_area: Option<Area>,
    /// Total nature-oriented area off-site
    pub total_nature_off_site_area: Option<Area>,
    /// Total use of land
    pub total_use_of_land: Option<Area>,
}

impl From<Option<&PreviousLandUseSummary>> for TypedPreviousLandUseSummary {
    fn from(summary: Option<&PreviousLandUseSummary>) -> Self {
        let Some(summary) = summary else {
            return TypedPreviousLandUseSummary {
                total_sealed_area: None,
                total_nature_on_site_area: None,
                total_nature_off_site_area: None,
                total_use_of_land: None,
            };
        };

        let unit = summary.unit_for_area;
        TypedPreviousLandUseSummary {
            total_sealed_area: Some(Area::new(summary.total_sealed_area, unit)),
            total_nature_on_site_area: Some(Area::new(summary.total_nature_on_site_area, unit)),
            total_nature_off_site_area: Some(Area::new(summary.total_nature_off_site_area, unit)),
            total_use_of_land: Some(Area::new(summary.total_use_of_land, unit)),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiteLandUseRow {
    /// Location/site name from the location name field
    pub location: String,
    /// Land-use type: `site`, `natureOnSite`, or `natureOffSite`
    pub land_use_type: SiteSpecification,
    /// Area in square meters (for reference)
    pub area: SquareMeter,
    /// Sealed area in square meters
    pub sealed_area: SquareMeter,
}

#[derive(Serialize, Debug, Clone, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SiteLandUseRowOutput {
    /// Location/site name from the location name field
    pub location: String,
    /// Land-use type: `site`, `natureOnSite`, or `natureOffSite`
    pub land_use_type: SiteSpecification,
    /// Area in square meters (for reference)
    pub area: Area,
    /// Sealed area in square meters
    pub sealed_area: Area,
}

pub fn site_land_use_row_to_output(
    row: SiteLandUseRow,
    unit_for_area: UnitForArea,
) -> SiteLandUseRowOutput {
    SiteLandUseRowOutput {
        location: row.location,
        land_use_type: row.land_use_type,
        area: Area::from_square_meters(row.area, unit_for_area),
        sealed_area: Area::from_square_meters(row.sealed_area, unit_for_area),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SiteSpecification {
    Site,
    NatureOnSite,
    NatureOffSite,
}

impl SiteSpecification {
    pub const EXPECTED: &'static str = "site, natureOnSite, or natureOffSite";
}

impl FromStr for SiteSpecification {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "site" => Ok(SiteSpecification::Site),
            "natureonsite" => Ok(SiteSpecification::NatureOnSite),
            "natureoffsite" => Ok(SiteSpecification::NatureOffSite),
            other => Err(anyhow::anyhow!(
                "Unrecognized site specification `{other}` (expected {expected})",
                expected = SiteSpecification::EXPECTED
            )),
        }
    }
}

pub fn summary_to_data_resource(
    site_rows: LandUseSummary,
    unit_for_area: UnitForArea,
) -> DataResource<Vec<LandUseSummaryRowOutput>> {
    DataResource {
        name: "Land Use".to_string(),
        schema: Fields {
            fields: vec![
                TableSchemaField {
                    name: "landUseType".into(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Land-use type".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "previousYear".into(),
                    r#type: Some(TableSchemaType::Number),
                    title: Some(format!("Previous year ({unit_for_area})")),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "reportingYear".into(),
                    r#type: Some(TableSchemaType::Number),
                    title: Some(format!("Reporting year ({unit_for_area})")),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "percentageChange".into(),
                    r#type: Some(TableSchemaType::Number),
                    title: Some("% change".into()),
                    ..Default::default()
                },
            ],
            primary_key: vec!["landUseType".to_string()].into(),
        },
        data: vec![
            land_use_summary_row_to_output(site_rows.total_sealed_area, unit_for_area),
            land_use_summary_row_to_output(site_rows.total_nature_on_site_area, unit_for_area),
            land_use_summary_row_to_output(site_rows.total_nature_off_site_area, unit_for_area),
            land_use_summary_row_to_output(site_rows.total_use_of_land, unit_for_area),
        ],
    }
}

pub fn site_to_data_resource(
    site_rows: Vec<SiteLandUseRow>,
    unit_for_area: UnitForArea,
) -> DataResource<Vec<SiteLandUseRowOutput>> {
    DataResource {
        name: "Site Land Use".to_string(),
        schema: Fields {
            fields: vec![
                TableSchemaField {
                    name: "location".into(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Location".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "landUseType".into(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Land-use type".into()),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "area".into(),
                    r#type: Some(TableSchemaType::Number),
                    title: Some(format!("Area ({unit_for_area})")),
                    ..Default::default()
                },
                TableSchemaField {
                    name: "sealedArea".into(),
                    r#type: Some(TableSchemaType::Number),
                    title: Some(format!("Sealed area ({unit_for_area})")),
                    ..Default::default()
                },
            ],
            primary_key: vec!["location".to_string()].into(),
        },
        data: site_rows
            .into_iter()
            .map(|row| site_land_use_row_to_output(row, unit_for_area))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_land_use_summary_types() {
        assert_eq!(
            LandUseSummaryRowType::TotalSealedArea.to_display_str(),
            "Total sealed area"
        );
        assert_eq!(
            LandUseSummaryRowType::TotalNatureOnSiteArea.to_display_str(),
            "Total nature-oriented area on-site"
        );
        assert_eq!(
            LandUseSummaryRowType::TotalNatureOffSiteArea.to_display_str(),
            "Total nature-oriented area off-site"
        );
        assert_eq!(
            LandUseSummaryRowType::TotalUseOfLand.to_display_str(),
            "Total use of land"
        );
    }

    #[test]
    fn it_converts_land_use_summary_row_to_output() {
        let unit = UnitForArea::Hectare;

        // With previous year
        let row = LandUseSummaryRow {
            land_use_type: LandUseSummaryRowType::TotalSealedArea,
            previous_year: Some(SquareMeter(10000.0)),
            reporting_year: SquareMeter(15000.0),
            percentage_change: Some(50.0),
        };
        let output = land_use_summary_row_to_output(row, unit);
        assert_eq!(output.land_use_type, "Total sealed area");
        assert!(output.previous_year.is_some());
        assert!(output.percentage_change.is_some());

        // Without previous year
        let row = LandUseSummaryRow {
            land_use_type: LandUseSummaryRowType::TotalUseOfLand,
            previous_year: None,
            reporting_year: SquareMeter(5000.0),
            percentage_change: None,
        };
        let output = land_use_summary_row_to_output(row, unit);
        assert_eq!(output.land_use_type, "Total use of land");
        assert!(output.previous_year.is_none());
        assert!(output.percentage_change.is_none());
    }

    #[test]
    fn it_converts_previous_land_use_summary_to_typed() {
        let unit = UnitForArea::SquareMeter;

        // With all values
        let summary = Some(&PreviousLandUseSummary {
            total_sealed_area: 1000.0,
            total_nature_on_site_area: 2000.0,
            total_nature_off_site_area: 3000.0,
            total_use_of_land: 4000.0,
            unit_for_area: unit,
        });
        let typed = TypedPreviousLandUseSummary::from(summary);
        assert!(typed.total_sealed_area.is_some());
        assert!(typed.total_nature_on_site_area.is_some());

        // None case
        let typed = TypedPreviousLandUseSummary::from(None);
        assert!(typed.total_sealed_area.is_none());
        assert!(typed.total_nature_on_site_area.is_none());
        assert!(typed.total_nature_off_site_area.is_none());
        assert!(typed.total_use_of_land.is_none());
    }

    #[test]
    fn it_parses_site_specifications() {
        // Valid cases with different casings
        assert!(matches!(
            SiteSpecification::from_str("site"),
            Ok(SiteSpecification::Site)
        ));
        assert!(matches!(
            SiteSpecification::from_str("Site"),
            Ok(SiteSpecification::Site)
        ));
        assert!(matches!(
            SiteSpecification::from_str("natureonsite"),
            Ok(SiteSpecification::NatureOnSite)
        ));
        assert!(matches!(
            SiteSpecification::from_str("NatureOnSite"),
            Ok(SiteSpecification::NatureOnSite)
        ));
        assert!(matches!(
            SiteSpecification::from_str("natureoffsite"),
            Ok(SiteSpecification::NatureOffSite)
        ));

        // Invalid cases
        let result = SiteSpecification::from_str("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unrecognized"));
    }

    #[test]
    fn it_creates_data_resources() {
        let unit = UnitForArea::Hectare;

        // Test summary_to_data_resource
        let summary = LandUseSummary {
            total_sealed_area: LandUseSummaryRow {
                land_use_type: LandUseSummaryRowType::TotalSealedArea,
                previous_year: Some(SquareMeter(100.0)),
                reporting_year: SquareMeter(150.0),
                percentage_change: Some(50.0),
            },
            total_nature_on_site_area: LandUseSummaryRow {
                land_use_type: LandUseSummaryRowType::TotalNatureOnSiteArea,
                previous_year: None,
                reporting_year: SquareMeter(50.0),
                percentage_change: None,
            },
            total_nature_off_site_area: LandUseSummaryRow {
                land_use_type: LandUseSummaryRowType::TotalNatureOffSiteArea,
                previous_year: None,
                reporting_year: SquareMeter(25.0),
                percentage_change: None,
            },
            total_use_of_land: LandUseSummaryRow {
                land_use_type: LandUseSummaryRowType::TotalUseOfLand,
                previous_year: None,
                reporting_year: SquareMeter(225.0),
                percentage_change: None,
            },
        };
        let resource = summary_to_data_resource(summary, unit);
        assert_eq!(resource.name, "Land Use");
        assert_eq!(resource.data.len(), 4);
        assert_eq!(resource.schema.fields.len(), 4);

        // Test site_to_data_resource
        let site_rows = vec![
            SiteLandUseRow {
                location: "Site A".to_string(),
                land_use_type: SiteSpecification::Site,
                area: SquareMeter(1000.0),
                sealed_area: SquareMeter(500.0),
            },
            SiteLandUseRow {
                location: "Site B".to_string(),
                land_use_type: SiteSpecification::NatureOnSite,
                area: SquareMeter(2000.0),
                sealed_area: SquareMeter(100.0),
            },
        ];
        let resource = site_to_data_resource(site_rows, unit);
        assert_eq!(resource.name, "Site Land Use");
        assert_eq!(resource.data.len(), 2);
        assert_eq!(resource.schema.fields.len(), 4);
    }
}
