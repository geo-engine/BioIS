use approx::AbsDiffEq;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Deref, Div, Mul, Sub};
use utoipa::ToSchema;

use crate::processes::parameters::{TableSchemaType, data_resource::HasTableSchemaType};

/// Area in square meters
#[derive(
    Deserialize, Serialize, Debug, PartialEq, AbsDiffEq, JsonSchema, ToSchema, Copy, Clone,
)]
pub struct SquareMeter(#[schemars(range(min = 0.0))] pub f64);

impl From<SquareMeter> for Hectare {
    fn from(sqm: SquareMeter) -> Self {
        Self(sqm.0 / 10_000.)
    }
}

impl From<Hectare> for SquareMeter {
    fn from(ha: Hectare) -> Self {
        Self(ha.0 * 10_000.)
    }
}

impl Deref for SquareMeter {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add for SquareMeter {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl AddAssign for SquareMeter {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Sub for SquareMeter {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl Mul<f64> for SquareMeter {
    type Output = SquareMeter;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div<SquareMeter> for SquareMeter {
    type Output = f64;

    fn div(self, rhs: SquareMeter) -> Self::Output {
        self.0 / rhs.0
    }
}

/// Area in hectares
#[derive(
    Deserialize, Serialize, Debug, PartialEq, AbsDiffEq, JsonSchema, ToSchema, Copy, Clone,
)]
pub struct Hectare(#[schemars(range(min = 0.0))] pub f64);

impl From<f64> for Hectare {
    fn from(ha: f64) -> Self {
        Hectare(ha)
    }
}

/// Year of reporting or change (e.g., 2023, 2024, etc.)
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone, PartialEq)]
#[serde(transparent)]
#[schemars(example = Year(2020))]
pub struct Year(#[schemars(range(min = 2000, max = 2100))] pub u16);

impl Year {
    pub fn new(year: u16) -> Self {
        Year(year)
    }
}

impl std::fmt::Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Length of a time window in years (e.g., 5 years).
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone, PartialEq)]
#[serde(transparent)]
#[schemars(example = YearRange(20))]
pub struct YearRange(#[schemars(range(min = 5, max = 30))] pub u16);

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
#[serde(transparent)]
#[schemars(example = Month(1))]
pub struct Month(#[schemars(range(min = 1, max = 12))] pub u8);

/// Distance in kilometers
#[derive(
    Deserialize, Serialize, Debug, PartialEq, AbsDiffEq, JsonSchema, ToSchema, Copy, Clone,
)]
#[serde(transparent)]
pub struct Kilometers(#[schemars(range(min = 0.0))] pub f64);

impl std::fmt::Display for Kilometers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} km", self.0)
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, JsonSchema, ToSchema)]
#[schemars(example = UnitForArea::Hectare, title = "Unit for area values")]
pub enum UnitForArea {
    #[serde(rename = "ha")]
    Hectare,
    #[serde(rename = "m²")]
    SquareMeter,
}

impl std::fmt::Display for UnitForArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitForArea::Hectare => write!(f, "ha"),
            UnitForArea::SquareMeter => write!(f, "m²"),
        }
    }
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, AbsDiffEq, JsonSchema, ToSchema)]
#[serde(untagged)] // NOTE: Deserialization would be ambiguous if we had a tagged enum, because both variants are numbers.
pub enum Area {
    Hectare(Hectare),
    SquareMeter(SquareMeter),
}

impl Area {
    pub fn new(value: f64, unit: UnitForArea) -> Self {
        match unit {
            UnitForArea::Hectare => Area::Hectare(Hectare(value)),
            UnitForArea::SquareMeter => Area::SquareMeter(SquareMeter(value)),
        }
    }

    pub fn from_square_meters(value: SquareMeter, unit: UnitForArea) -> Self {
        match unit {
            UnitForArea::Hectare => Area::Hectare(value.into()),
            UnitForArea::SquareMeter => Area::SquareMeter(value),
        }
    }

    pub fn to_square_meters(self) -> SquareMeter {
        match self {
            Area::Hectare(ha) => ha.into(),
            Area::SquareMeter(sqm) => sqm,
        }
    }

    pub fn unit(&self) -> UnitForArea {
        match self {
            Area::Hectare(_) => UnitForArea::Hectare,
            Area::SquareMeter(_) => UnitForArea::SquareMeter,
        }
    }

    pub fn value(&self) -> f64 {
        match self {
            Area::Hectare(Hectare(v)) | Area::SquareMeter(SquareMeter(v)) => *v,
        }
    }
}

/// Percentage value, e.g. of change. Stores it as the plain value, e.g. 0.23 for 23%, but serializes it as a string with a percent sign, e.g. "23 %".
#[derive(Debug, JsonSchema, ToSchema, Copy, Clone, PartialEq, AbsDiffEq)]
#[schemars(example = Percentage(23.0))]
pub struct Percentage(f64);

impl Percentage {
    /// Creates a Percentage from a whole number (e.g., 23.0 for 23%)
    pub fn from_percent(percent: f64) -> Self {
        Self(percent / 100.0)
    }

    /// Creates a Percentage directly from a fraction (e.g., 0.23 for 23%)
    pub fn from_fraction(fraction: f64) -> Self {
        Self(fraction)
    }

    /// Returns the normalized decimal fraction (0.23) for math operations
    pub fn as_fraction(self) -> f64 {
        self.0
    }

    /// Returns the percentage scalar (23.0) for display/logging
    pub fn as_percent(self) -> f64 {
        self.0 * 100.0
    }

    /// Helper to apply the percentage directly to a value
    pub fn apply_to(self, value: f64) -> f64 {
        value * self.0
    }
}

impl Serialize for Percentage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = self.as_percent();
        let mut formatted = format!("{value:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        formatted.push_str(" %");
        serializer.serialize_str(&formatted)
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Visitor;

        struct PercentageVisitor;

        impl Visitor<'_> for PercentageVisitor {
            type Value = Percentage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a percentage value as a string like '23 %' or a number")
            }

            fn visit_str<E>(self, value: &str) -> Result<Percentage, E>
            where
                E: serde::de::Error,
            {
                // Remove leading/trailing whitespace
                let trimmed = value.trim();
                // Remove trailing % and whitespace
                let without_percent = trimmed
                    .trim_end_matches(|c: char| c == '%' || c.is_whitespace())
                    .trim_end();

                without_percent
                    .parse::<f64>()
                    .map(Percentage::from_percent)
                    .map_err(|_| E::custom("expected a number followed by optional %"))
            }

            fn visit_string<E>(self, value: String) -> Result<Percentage, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Percentage, E>
            where
                E: serde::de::Error,
            {
                Ok(Percentage::from_percent(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Percentage, E>
            where
                E: serde::de::Error,
            {
                Ok(Percentage::from_percent(value as f64))
            }
        }

        deserializer.deserialize_any(PercentageVisitor)
    }
}

impl Deref for Percentage {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl HasTableSchemaType for Percentage {
    fn table_schema_type() -> TableSchemaType {
        TableSchemaType::String // Represented as a string with a percent sign in the table schema
    }
}

mod db {
    use super::*;
    use diesel::{
        deserialize::{self, FromSql},
        pg::{Pg, PgValue},
        sql_types::Double,
    };

    impl FromSql<Double, Pg> for Hectare {
        fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
            f64::from_sql(value).map(Hectare)
        }
    }

    impl FromSql<Double, Pg> for SquareMeter {
        fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
            f64::from_sql(value).map(SquareMeter)
        }
    }

    impl FromSql<Double, Pg> for Kilometers {
        fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
            f64::from_sql(value).map(Kilometers)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_performs_square_meter_arithmetic_operations() {
        assert_eq!(SquareMeter(100.0) + SquareMeter(50.0), SquareMeter(150.0));

        let mut a = SquareMeter(100.0);
        a += SquareMeter(50.0);
        assert_eq!(a, SquareMeter(150.0));

        assert_eq!(SquareMeter(100.0) - SquareMeter(30.0), SquareMeter(70.0));
        assert_eq!(SquareMeter(50.0) * 2.0, SquareMeter(100.0));
        assert_abs_diff_eq!(SquareMeter(100.0) / SquareMeter(4.0), 25.0);
        assert_abs_diff_eq!(*SquareMeter(42.5), 42.5);
    }

    #[test]
    fn it_converts_between_square_meter_and_hectare() {
        assert_eq!(Hectare::from(SquareMeter(10_000.0)), Hectare(1.0));
        let result: SquareMeter = Hectare(2.5).into();
        assert_eq!(result, SquareMeter(25_000.0));

        let sqm = SquareMeter(12_345.678_9);
        let ha: Hectare = sqm.into();
        let sqm_back: SquareMeter = ha.into();
        assert_abs_diff_eq!(sqm_back, sqm, epsilon = 0.0001);
    }

    #[test]
    fn it_constructs_and_serializes_hectare() {
        assert_eq!(Hectare::from(3.5), Hectare(3.5));
        assert_eq!(serde_json::to_string(&Hectare(2.5)).unwrap(), "2.5");
        assert_eq!(
            serde_json::from_str::<Hectare>("1.75").unwrap(),
            Hectare(1.75)
        );
    }

    #[test]
    fn it_displays_and_serializes_year() {
        assert_eq!(Year(2023).to_string(), "2023");
        assert_eq!(serde_json::to_string(&Year(2024)).unwrap(), "2024");
        assert_eq!(serde_json::from_str::<Year>("2020").unwrap().0, 2020);
    }

    #[test]
    fn it_serializes_month() {
        assert_eq!(serde_json::to_string(&Month(6)).unwrap(), "6");
        assert_eq!(serde_json::from_str::<Month>("12").unwrap().0, 12);
    }

    #[test]
    fn it_displays_and_serializes_kilometers() {
        assert_eq!(Kilometers(42.5).to_string(), "42.5 km");
        assert_eq!(Kilometers(0.0).to_string(), "0 km");
        assert_eq!(serde_json::to_string(&Kilometers(10.5)).unwrap(), "10.5");
        assert_eq!(
            serde_json::from_str::<Kilometers>("5.25").unwrap(),
            Kilometers(5.25)
        );
    }

    #[test]
    fn it_displays_and_serializes_unit_for_area() {
        assert_eq!(UnitForArea::Hectare.to_string(), "ha");
        assert_eq!(UnitForArea::SquareMeter.to_string(), "m²");
        assert_eq!(
            serde_json::to_string(&UnitForArea::Hectare).unwrap(),
            "\"ha\""
        );
        assert_eq!(
            serde_json::to_string(&UnitForArea::SquareMeter).unwrap(),
            "\"m²\""
        );
        assert_eq!(
            serde_json::from_str::<UnitForArea>("\"ha\"").unwrap(),
            UnitForArea::Hectare
        );
        assert_eq!(
            serde_json::from_str::<UnitForArea>("\"m²\"").unwrap(),
            UnitForArea::SquareMeter
        );
    }

    #[test]
    fn it_constructs_area() {
        assert_eq!(
            Area::new(5.0, UnitForArea::Hectare),
            Area::Hectare(Hectare(5.0))
        );
        assert_eq!(
            Area::new(50_000.0, UnitForArea::SquareMeter),
            Area::SquareMeter(SquareMeter(50_000.0))
        );
        assert_eq!(
            Area::from_square_meters(SquareMeter(20_000.0), UnitForArea::Hectare),
            Area::Hectare(Hectare(2.0))
        );
        assert_eq!(
            Area::from_square_meters(SquareMeter(15_000.0), UnitForArea::SquareMeter),
            Area::SquareMeter(SquareMeter(15_000.0))
        );
    }

    #[test]
    fn it_converts_area_to_square_meters() {
        assert_eq!(
            Area::Hectare(Hectare(3.0)).to_square_meters(),
            SquareMeter(30_000.0)
        );
        assert_eq!(
            Area::SquareMeter(SquareMeter(12_500.0)).to_square_meters(),
            SquareMeter(12_500.0)
        );
    }

    #[test]
    fn it_accesses_area_properties() {
        assert_eq!(Area::Hectare(Hectare(1.5)).unit(), UnitForArea::Hectare);
        assert_abs_diff_eq!(Area::Hectare(Hectare(1.5)).value(), 1.5);
        assert_eq!(
            Area::SquareMeter(SquareMeter(15_000.0)).unit(),
            UnitForArea::SquareMeter
        );
        assert_abs_diff_eq!(Area::SquareMeter(SquareMeter(15_000.0)).value(), 15_000.0);
    }

    #[test]
    fn it_serializes_area() {
        assert_eq!(
            serde_json::to_string(&Area::Hectare(Hectare(1.5))).unwrap(),
            "1.5"
        );
    }

    #[test]
    fn it_handles_edge_cases_with_zero_large_and_fractional_values() {
        assert_eq!(Hectare::from(SquareMeter(0.0)), Hectare(0.0));
        assert_eq!(
            Area::new(0.0, UnitForArea::SquareMeter).to_square_meters(),
            SquareMeter(0.0)
        );

        assert_abs_diff_eq!(
            Hectare::from(SquareMeter(1_000_000_000.0)),
            Hectare(100_000.0)
        );
        assert_abs_diff_eq!(
            Area::new(1_000_000.0, UnitForArea::Hectare).to_square_meters(),
            SquareMeter(10_000_000_000.0),
            epsilon = 1.0
        );

        let sqm = SquareMeter(12_345.678_9);
        let ha: Hectare = sqm.into();
        let sqm_back: SquareMeter = ha.into();
        assert_abs_diff_eq!(sqm_back, sqm, epsilon = 0.0001);
    }

    #[test]
    fn it_serializes_and_deserializes_percentage_as_string_with_percent_sign() {
        // Test integer-like value
        let percentage = Percentage::from_percent(23.0);
        let serialized = serde_json::to_string(&percentage).unwrap();
        assert_eq!(serialized, "\"23 %\"");

        let deserialized: Percentage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, Percentage::from_percent(23.0));

        // Test with one decimal
        let percentage_decimal = Percentage::from_percent(42.5);
        let serialized_decimal = serde_json::to_string(&percentage_decimal).unwrap();
        assert_eq!(serialized_decimal, "\"42.5 %\"");
        let deserialized_decimal: Percentage = serde_json::from_str(&serialized_decimal).unwrap();
        assert_eq!(deserialized_decimal, Percentage::from_percent(42.5));

        // Test with two decimals
        let percentage_two_decimals = Percentage::from_percent(33.33);
        let serialized_two = serde_json::to_string(&percentage_two_decimals).unwrap();
        assert_eq!(serialized_two, "\"33.33 %\"");

        // Test truncation to 2 decimal places
        let percentage_many_decimals = Percentage::from_percent(42.12345);
        let serialized_truncated = serde_json::to_string(&percentage_many_decimals).unwrap();
        assert_eq!(serialized_truncated, "\"42.12 %\"");

        // Test deserialization with various formats
        assert_eq!(
            serde_json::from_str::<Percentage>("\"15 %\"").unwrap(),
            Percentage::from_percent(15.0)
        );
        assert_eq!(
            serde_json::from_str::<Percentage>("\"15%\"").unwrap(),
            Percentage::from_percent(15.0)
        );
        assert_eq!(
            serde_json::from_str::<Percentage>("\" 15 % \"").unwrap(),
            Percentage::from_percent(15.0)
        );
    }
}
