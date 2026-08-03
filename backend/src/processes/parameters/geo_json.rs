use anyhow::Result;
use geojson::{Feature, FeatureCollection, PointType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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

impl TryFrom<geoengine_api_client::models::GeoJson> for GeoJsonFeatureCollection {
    type Error = anyhow::Error;

    fn try_from(value: geoengine_api_client::models::GeoJson) -> Result<Self, Self::Error> {
        if value.r#type != geoengine_api_client::models::CollectionType::FeatureCollection {
            return Err(anyhow::anyhow!("GeoJSON is not a FeatureCollection"));
        }

        let feature_collection = FeatureCollection {
            bbox: None,
            features: value
                .features
                .into_iter()
                .map(serde_json::from_value::<geojson::Feature>)
                .collect::<Result<Vec<_>, _>>()?,
            foreign_members: None,
        };

        Ok(GeoJsonFeatureCollection(feature_collection))
    }
}

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

pub mod geojson_feature_utils {
    use super::*;

    /// Extracts the ID of a `GeoJSON` feature as a string, returning "unknown" if the ID is missing.
    pub fn id_str(feature: &Feature) -> String {
        feature
            .id
            .as_ref()
            .map_or("unknown".to_string(), |id| match id {
                geojson::feature::Id::String(s) => s.clone(),
                geojson::feature::Id::Number(n) => n.to_string(),
            })
    }

    /// Extracts a string property from a `GeoJSON` feature, returning an error if the property is missing or not a string.
    pub fn get_str<'f>(feature: &'f Feature, field: &str) -> Result<&'f str> {
        let Some(value) = feature
            .properties
            .as_ref()
            .and_then(|props| props.get(field))
        else {
            return Err(anyhow::anyhow!(
                "Feature `{id}` is missing property `{field}`",
                id = id_str(feature),
                field = field
            ));
        };
        let Some(value_str) = value.as_str() else {
            return Err(anyhow::anyhow!(
                "Feature `{id}` property `{field}` is not a string",
                id = id_str(feature),
                field = field
            ));
        };
        Ok(value_str)
    }

    /// Extracts a string property from a `GeoJSON` feature, returning an error if the property is missing or not a string.
    pub fn get_string(feature: &Feature, field: &str) -> Result<String> {
        get_str(feature, field).map(ToString::to_string)
    }

    /// Extracts a numeric property from a `GeoJSON` feature, returning an error if the property is missing or not a number.
    pub fn get_number(feature: &Feature, field: &str) -> Result<f64> {
        let Some(value) = feature
            .properties
            .as_ref()
            .and_then(|props| props.get(field))
        else {
            return Err(anyhow::anyhow!(
                "Feature `{id}` is missing property `{field}`",
                id = id_str(feature),
                field = field
            ));
        };
        let Some(value_num) = value.as_f64() else {
            return Err(anyhow::anyhow!(
                "Feature `{id}` property `{field}` is not a number",
                id = id_str(feature),
                field = field
            ));
        };
        Ok(value_num)
    }

    pub fn check_property_is_string(feature: &Feature, field: &str) -> Result<()> {
        let Some(value) = feature
            .properties
            .as_ref()
            .and_then(|props| props.get(field))
        else {
            return Err(anyhow::anyhow!(
                "Feature `{id}` is missing property `{field}`",
                id = id_str(feature),
                field = field
            ));
        };
        if !value.is_string() {
            return Err(anyhow::anyhow!(
                "Feature `{id}` property `{field}` is not a string",
                id = id_str(feature),
                field = field
            ));
        }
        Ok(())
    }
}

pub mod geojson_feature_collection_utils {
    use super::*;

    /// Returns the name of the `GeoJsonFeatureCollection` if it has a `name` foreign member, otherwise returns `None`.
    pub fn name(feature_collection: &FeatureCollection) -> Option<&str> {
        get_foreign_member_string(feature_collection, "name").ok()
    }

    pub fn get_foreign_member_string<'f>(
        feature_collection: &'f FeatureCollection,
        member_name: &str,
    ) -> Result<&'f str> {
        let Some(foreign_members) = &feature_collection.foreign_members else {
            return Err(anyhow::anyhow!(
                "FeatureCollection is missing foreign members, cannot get `{member_name}`",
            ));
        };

        foreign_members
            .get(member_name)
            .and_then(|name| name.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("FeatureCollection is missing foreign member `{member_name}`")
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use geojson_feature_collection_utils::*;
    use geojson_feature_utils::*;
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
    fn it_extracts_feature_ids() {
        // String ID
        let mut feature_string_id: Feature = serde_json::from_value(serde_json::json!({
            "type": "Feature",
            "id": "feature_123",
            "geometry": null,
            "properties": null
        }))
        .unwrap();
        assert_eq!(id_str(&feature_string_id), "feature_123");

        // Number ID
        feature_string_id.id = Some(geojson::feature::Id::Number(42i64.into()));
        assert_eq!(id_str(&feature_string_id), "42");

        // No ID
        feature_string_id.id = None;
        assert_eq!(id_str(&feature_string_id), "unknown");
    }

    #[test]
    fn it_extracts_and_validates_feature_properties() {
        let feature: Feature = serde_json::from_value(serde_json::json!({
            "type": "Feature",
            "id": "test_feature",
            "geometry": null,
            "properties": {
                "name": "Test",
                "value": 2.14
            }
        }))
        .unwrap();

        // Successful string extraction
        assert_eq!(get_str(&feature, "name").unwrap(), "Test");
        assert_eq!(get_string(&feature, "name").unwrap(), "Test");

        // Successful number extraction
        assert_abs_diff_eq!(get_number(&feature, "value").unwrap(), 2.14);

        // Missing property
        assert!(get_str(&feature, "missing").is_err());
        assert!(get_number(&feature, "missing").is_err());

        // Wrong type (number as string, string as number)
        assert!(get_str(&feature, "value").is_err());
        assert!(get_number(&feature, "name").is_err());

        // Property validation
        assert!(check_property_is_string(&feature, "name").is_ok());
        assert!(check_property_is_string(&feature, "value").is_err());
        assert!(check_property_is_string(&feature, "missing").is_err());
    }

    #[test]
    fn it_handles_feature_collection_foreign_members() {
        // With foreign members including name
        let mut fc: FeatureCollection = serde_json::from_value(serde_json::json!({
            "type": "FeatureCollection",
            "features": [],
            "name": "Test Collection"
        }))
        .unwrap();

        assert_eq!(name(&fc), Some("Test Collection"));
        assert_eq!(
            get_foreign_member_string(&fc, "name").unwrap(),
            "Test Collection"
        );

        // Missing foreign members
        fc.foreign_members = None;
        assert_eq!(name(&fc), None);
        assert!(get_foreign_member_string(&fc, "name").is_err());

        // Foreign members present but name missing
        fc.foreign_members = Some(
            serde_json::json!({"other": "value"})
                .as_object()
                .unwrap()
                .clone(),
        );
        assert_eq!(name(&fc), None);
        assert!(get_foreign_member_string(&fc, "name").is_err());
    }
}
