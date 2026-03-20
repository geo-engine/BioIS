use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(transparent)]
pub struct Coordinate(pub [f64; 2]);

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
