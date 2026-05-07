use geoengine_api_client::models::SpatialPartition2D;
use geojson::{FeatureCollection, PointType};
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

pub trait ToBbox {
    fn to_bbox(&self, buffer: f64) -> SpatialPartition2D;
}

impl ToBbox for PointType {
    fn to_bbox(&self, buffer: f64) -> SpatialPartition2D {
        use geoengine_api_client::models::Coordinate2D;

        let [x, y] = self.as_slice() else {
            debug_assert!(false, "Expected PointType to have exactly 2 coordinates");
            return SpatialPartition2D::new(
                Coordinate2D::new(0.0, 0.0),
                Coordinate2D::new(0.0, 0.0),
            );
        };
        SpatialPartition2D::new(
            Coordinate2D::new(x + buffer, y - buffer),
            Coordinate2D::new(x - buffer, y + buffer),
        )
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PointGeoJsonInput {
    #[schema(inline)]
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
#[derive(Deserialize, Serialize, Debug, PartialEq, JsonSchema, ToSchema, Copy, Clone)]
pub struct Hectare(pub f64);

impl From<f64> for Hectare {
    fn from(ha: f64) -> Self {
        Hectare(ha)
    }
}

/// Area in square meters
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
pub struct SquareMeter(pub f64);

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
pub struct Year(pub u16);

/// Distance in kilometers
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
#[serde(transparent)]
pub struct Kilometers(pub f64);

/// Documentation source for audit and provenance, e.g. a Geo Engine workflow or a scientific paper.
/// This is included in the outputs of the process for traceability and auditing purposes.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct DocumentationSource {
    /// A human-readable identifier of the documentation source (e.g. "Geo Engine workflow XYZ")
    pub data: String,
    /// A description, citation or URL pointing to the source of the documentation (e.g. a link to a Geo Engine workflow, or a scientific paper)
    pub documentation_source: String,
}

/// A property of the input data that is relevant for the process, e.g. a property field in a input `GeoJSON`.
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, format = "relative-json-pointer")]
#[schemars(transform = relative_json_pointer_format)]
pub struct RelativeJsonPointer(String);

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
}
