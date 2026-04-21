use geoengine_api_client::models::SpatialPartition2D;
use geojson::{PointType, PolygonType};
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
                    "reference": $url.to_string(),
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
    PolygonGeoJson,
    "GeoJSON Polygon",
    "https://geojson.org/schema/Polygon.json"
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
    pub value: PointGeoJson,
    pub media_type: GeoJsonInputMediaType,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum GeoJsonInputMediaType {
    #[serde(rename = "application/geo+json")]
    GeoJson,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PointGeoJson {
    pub r#type: PointGeoJsonType,
    pub coordinates: PointType,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub enum PointGeoJsonType {
    Point,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PolygonGeoJsonInput {
    pub value: PolygonGeoJson,
    pub media_type: GeoJsonInputMediaType,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PolygonGeoJson {
    pub r#type: PolygonGeoJsonType,
    pub coordinates: PolygonType,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub enum PolygonGeoJsonType {
    Polygon,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub enum FeatureGeoJsonType {
    Feature,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub enum FeatureCollectionGeoJsonType {
    FeatureCollection,
}

/// A GeoJSON FeatureCollection containing only Polygon features.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct GeoJsonPolygonFeatureCollection {
    pub r#type: FeatureCollectionGeoJsonType,
    pub features: Vec<GeoJsonPolygonFeature>,
}

/// A GeoJSON Feature containing polygon geometry.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct GeoJsonPolygonFeature {
    pub r#type: FeatureGeoJsonType,
    pub geometry: PolygonGeoJson,
}

/// Area in hectares
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Copy, Clone)]
pub struct Hectare(pub f64);

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
pub struct Year(pub u16);

#[cfg(test)]
mod tests {
    use super::*;

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
                    }
                }
            ]
        });

        let polygon_feature_collection: GeoJsonPolygonFeatureCollection =
            serde_json::from_value(polygon_feature_collection_json.clone())
                .expect("Failed to parse Polygon GeoJSON");
        assert_eq!(polygon_feature_collection.features.len(), 1);

        assert_eq!(
            serde_json::to_value(&polygon_feature_collection).unwrap(),
            polygon_feature_collection_json
        );
    }
}
