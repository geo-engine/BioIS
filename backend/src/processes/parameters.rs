use geoengine_openapi_client::models::SpatialPartition2D;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(transparent)]
pub struct Coordinate(pub [f64; 2]);

pub trait ToBbox {
    fn to_bbox(&self, buffer: f64) -> SpatialPartition2D;
}

impl ToBbox for Coordinate {
    fn to_bbox(&self, buffer: f64) -> SpatialPartition2D {
        use geoengine_openapi_client::models::Coordinate2D;

        let [x, y] = self.0;
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
    pub media_type: PointGeoJsonInputMediaType,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum PointGeoJsonInputMediaType {
    #[serde(rename = "application/geo+json")]
    GeoJson,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PointGeoJson {
    pub r#type: PointGeoJsonType,
    pub coordinates: Coordinate,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub enum PointGeoJsonType {
    Point,
}
