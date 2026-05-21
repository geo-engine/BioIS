use crate::processes::util::json_input_value;
use approx::AbsDiffEq;
use geojson::{FeatureCollection, PointType};
use ogcapi::types::processes::InputValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A macro to implement both `schemars::JsonSchema` and `utoipa::ToSchema` for external types by referencing an external schema URL.
macro_rules! impl_extern_schema {
    ($target:ty, $name:expr, $url:expr) => {
        // --- Schemars Implementation ---
        impl schemars::JsonSchema for $target {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed($name)
            }
            fn json_schema(_gen: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "$ref": $url.to_string(),
                })
            }
        }

        // --- Utoipa Implementation ---
        impl utoipa::PartialSchema for $target {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::Schema> {
                utoipa::openapi::RefOr::Ref(utoipa::openapi::Ref::new($url))
            }
        }
        impl utoipa::ToSchema for $target {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed($url)
            }
        }
    };

    ($target:ty, $name:expr, $url:expr, $($generics:tt)*) => {
        // --- Schemars Implementation ---
        impl <$($generics)*> schemars::JsonSchema for $target {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed($name)
            }
            fn json_schema(_gen: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "$ref": $url.to_string(),
                })
            }
        }

        // --- Utoipa Implementation ---
        impl <$($generics)*> utoipa::PartialSchema for $target {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::Schema> {
                utoipa::openapi::RefOr::Ref(utoipa::openapi::Ref::new($url))
            }
        }
        impl <$($generics)*> utoipa::ToSchema for $target {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed($url)
            }
        }
    };
}

impl_extern_schema!(
    PointGeoJson,
    "GeoJSON Point",
    "https://geojson.org/schema/Point.json"
);

impl_extern_schema!(
    GeoJsonFeatureCollection,
    "GeoJSON FeatureCollection",
    "https://geojson.org/schema/FeatureCollection.json"
);

impl_extern_schema!(
    DataResource<R>,
    "Data Resource",
    "https://specs.frictionlessdata.io/schemas/data-resource.json",
    R
);

impl_extern_schema!(
    DataResourceSchema,
    "Data Resource",
    "https://specs.frictionlessdata.io/schemas/data-resource.json",
);

/// Cf. <https://github.com/juhaku/utoipa/issues/1346>
#[derive(Debug)]
pub struct DataResourceSchema;

/// Data resources for outputting tabular data with JSON.
/// Based on <https://specs.frictionlessdata.io/schemas/data-resource.json>.
#[derive(Serialize, Debug)]
pub struct DataResource<R> {
    pub data: R,
    pub schema: Fields,
}

impl<R: Serialize> DataResource<R> {
    pub fn to_input_value(&self) -> anyhow::Result<InputValue> {
        Ok(json_input_value(serde_json::to_value(self)?))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Fields {
    pub fields: Vec<TableSchemaField>,
}

/// Field specification for Table Schema, based on <https://specs.frictionlessdata.io/table-schema/>.
#[derive(Serialize, Deserialize, Debug)]
pub struct TableSchemaField {
    pub name: String,
    #[serde(default)]
    pub r#type: Option<TableSchemaType>,
    pub title: Option<String>,
    // TODO: more descriptors
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum TableSchemaType {
    #[default]
    String,
    Number,
    Integer,
    Boolean,
    // TODO: more types
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PointGeoJsonInput {
    #[schema(inline)]
    #[schemars(example = PointGeoJson {
        r#type: PointGeoJsonType::Point,
        coordinates: PointType::from((8.771_796, 50.808_453)),
    })]
    pub value: PointGeoJson,
    pub media_type: GeoJsonInputMediaType,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum GeoJsonInputMediaType {
    #[serde(rename = "application/geo+json")]
    GeoJson,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PointGeoJson {
    pub r#type: PointGeoJsonType,
    pub coordinates: PointType,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, ToSchema)]
pub enum PointGeoJsonType {
    Point,
}

/// A `GeoJSON` `FeatureCollection` containing only Polygon features.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GeoJsonFeatureCollection(FeatureCollection);

impl AsRef<FeatureCollection> for GeoJsonFeatureCollection {
    fn as_ref(&self) -> &FeatureCollection {
        &self.0
    }
}

impl From<FeatureCollection> for GeoJsonFeatureCollection {
    fn from(fc: FeatureCollection) -> Self {
        GeoJsonFeatureCollection(fc)
    }
}

/// A `GeoJSON` `FeatureCollection` input
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollectionGeoJsonInput {
    #[schema(inline)]
    pub value: GeoJsonFeatureCollection,
    pub media_type: GeoJsonInputMediaType,
}

impl FeatureCollectionGeoJsonInput {
    pub fn value(&self) -> &FeatureCollection {
        self.value.as_ref()
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
}

/// Area in square meters
#[derive(
    Deserialize, Serialize, Debug, PartialEq, AbsDiffEq, JsonSchema, ToSchema, Copy, Clone,
)]
pub struct SquareMeter(#[schemars(range(min = 0.0))] pub f64);

impl From<SquareMeter> for Hectare {
    fn from(sqm: SquareMeter) -> Self {
        Hectare(sqm.0 / 10_000.)
    }
}

impl From<Hectare> for SquareMeter {
    fn from(ha: Hectare) -> Self {
        SquareMeter(ha.0 * 10_000.)
    }
}

/// Year of reporting or change (e.g., 2023, 2024, etc.)
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
#[serde(transparent)]
#[allow(
    unused,
    reason = "Currently not used, but may be needed for future processes that require a year parameter"
)]
#[schemars(example = Year(2020))]
pub struct Year(#[schemars(range(min = 2000, max = 2100))] pub u16);

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
#[serde(transparent)]
#[schemars(example = Month(1))]
pub struct Month(#[schemars(range(min = 1, max = 12))] pub u8);

/// Distance in kilometers
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
#[serde(transparent)]
pub struct Kilometers(#[schemars(range(min = 0.0))] pub f64);

/// Documentation source for audit and provenance, e.g. a Geo Engine workflow or a scientific paper.
/// This is included in the outputs of the process for traceability and auditing purposes.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct DocumentationSource {
    /// A human-readable identifier of the documentation source (e.g. "Geo Engine workflow XYZ")
    pub data: String,
    /// A description, citation or URL pointing to the source of the documentation (e.g. a link to a Geo Engine workflow, or a scientific paper)
    pub documentation_source: String,
}

impl From<Vec<DocumentationSource>> for DataResource<Vec<DocumentationSource>> {
    fn from(value: Vec<DocumentationSource>) -> Self {
        Self {
            data: value,
            schema: Fields {
                fields: vec![
                    TableSchemaField {
                        name: "data".to_string(),
                        r#type: Some(TableSchemaType::String),
                        title: Some("Data".to_string()),
                    },
                    TableSchemaField {
                        name: "documentation_source".to_string(),
                        r#type: Some(TableSchemaType::String),
                        title: Some("Documentation Source".to_string()),
                    },
                ],
            },
        }
    }
}

/// A property of the input data that is relevant for the process, e.g. a property field in a input `GeoJSON`.
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, format = "relative-json-pointer")]
#[schemars(transform = relative_json_pointer_format)]
pub struct RelativeJsonPointer(#[schemars(length(min = 1))] pub String);

fn relative_json_pointer_format(schema: &mut schemars::Schema) {
    schema.insert("format".into(), "relative-json-pointer".into());
}

impl AsRef<str> for RelativeJsonPointer {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RelativeJsonPointer {
    fn from(s: &str) -> Self {
        RelativeJsonPointer(s.to_string())
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, JsonSchema, ToSchema)]
#[schemars(example = UnitForArea::Hectare)]
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, AbsDiffEq, JsonSchema, ToSchema)]
#[serde(untagged)]
pub enum Area {
    Hectare(Hectare),
    SquareMeter(SquareMeter),
}

impl Area {
    pub fn from_square_meters(value: SquareMeter, unit: UnitForArea) -> Self {
        match unit {
            UnitForArea::Hectare => Area::Hectare(value.into()),
            UnitForArea::SquareMeter => Area::SquareMeter(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_deserializes_geo_json() {
        let point_geometry_json: serde_json::Value = serde_json::json!({
            "type": "Point",
            "coordinates": [102.0, 0.5]
        });

        let point: PointGeoJson =
            serde_json::from_value(point_geometry_json.clone()).expect("Failed to parse GeoJSON");

        assert_eq!(serde_json::to_value(&point).unwrap(), point_geometry_json);

        let polygon_feature_collection_json = serde_json::json!({
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[102.0, 0.0], [103.0, 1.0], [104.0, 0.0], [102.0, 0.0]]]
                    },
                    "properties": null
                }
            ]
        });

        let polygon_feature_collection: GeoJsonFeatureCollection =
            serde_json::from_value(polygon_feature_collection_json.clone())
                .expect("Failed to parse Polygon GeoJSON");
        assert_eq!(polygon_feature_collection.0.features.len(), 1);

        assert_eq!(
            serde_json::to_value(&polygon_feature_collection).unwrap(),
            polygon_feature_collection_json
        );
    }

    #[test]
    fn it_serializes_area() {
        let area = Area::Hectare(Hectare(1.5));
        let serialized = serde_json::to_string(&area).unwrap();
        assert_eq!(serialized, "1.5");
    }
}
