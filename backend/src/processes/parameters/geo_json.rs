use anyhow::Result;
use geojson::{Feature, FeatureCollection, PointType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, marker::PhantomData};
use utoipa::{
    ToSchema,
    openapi::{
        AllOfBuilder, ArrayBuilder, ObjectBuilder, Ref, RefOr, Schema, schema::AnyOfBuilder,
    },
};

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

/// A `GeoJSON` `FeatureCollection`.
///
/// The type parameter `P` describes the expected feature properties and geometry types.
/// It only affects the generated JSON schema, (de)serialization is not restricted.
pub struct GeoJsonFeatureCollection<P = AnyFeatureProperties>(FeatureCollection, PhantomData<P>);

/// Unrestricted feature properties, e.g., for results from Geo Engine.
#[derive(Clone, Debug)]
pub enum AnyFeatureProperties {}

/// Describes the features of a [`GeoJsonFeatureCollection`] for its schema.
///
/// The implementing type is the schema of the feature properties.
pub trait FeatureProperties {
    /// The allowed geometry types of the features.
    const GEOMETRY_TYPES: &'static [GeoJsonGeometryType];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeoJsonGeometryType {
    Point,
    MultiPoint,
    Polygon,
    MultiPolygon,
}

impl GeoJsonGeometryType {
    const fn schema_url(self) -> &'static str {
        match self {
            GeoJsonGeometryType::Point => "https://geojson.org/schema/Point.json",
            GeoJsonGeometryType::MultiPoint => "https://geojson.org/schema/MultiPoint.json",
            GeoJsonGeometryType::Polygon => "https://geojson.org/schema/Polygon.json",
            GeoJsonGeometryType::MultiPolygon => "https://geojson.org/schema/MultiPolygon.json",
        }
    }
}

const FEATURE_COLLECTION_SCHEMA_URL: &str = "https://geojson.org/schema/FeatureCollection.json";

/// Removes the `required` keyword from a properties schema.
///
/// Feature properties are only hints for the expected properties,
/// since the actual property names are selected by [`super::RelativeJsonPointer`] inputs.
pub fn optional_feature_properties(schema: &mut schemars::Schema) {
    schema.remove("required");
}

impl<P> Clone for GeoJsonFeatureCollection<P> {
    fn clone(&self) -> Self {
        GeoJsonFeatureCollection(self.0.clone(), PhantomData)
    }
}

impl<P> std::fmt::Debug for GeoJsonFeatureCollection<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("GeoJsonFeatureCollection")
            .field(&self.0)
            .finish()
    }
}

impl<P> Serialize for GeoJsonFeatureCollection<P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, P> Deserialize<'de> for GeoJsonFeatureCollection<P> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        FeatureCollection::deserialize(deserializer).map(Self::from)
    }
}

impl<P: FeatureProperties + JsonSchema> JsonSchema for GeoJsonFeatureCollection<P> {
    fn schema_name() -> Cow<'static, str> {
        format!("GeoJsonFeatureCollection_{}", P::schema_name()).into()
    }

    fn inline_schema() -> bool {
        true
    }

    fn json_schema(generator: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
        let geometries = P::GEOMETRY_TYPES
            .iter()
            .map(|geometry_type| serde_json::json!({ "$ref": geometry_type.schema_url() }))
            .collect::<Vec<_>>();
        let properties = P::json_schema(generator);

        schemars::json_schema!({
            "allOf": [
                { "$ref": FEATURE_COLLECTION_SCHEMA_URL },
                {
                    "type": "object",
                    "properties": {
                        "features": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    // `anyOf` since the geometry types are exclusive anyway and
                                    // validators without the external schemas cannot distinguish them
                                    "geometry": { "anyOf": geometries },
                                    "properties": properties,
                                },
                            },
                        },
                    },
                },
            ],
        })
    }
}

// `ComposeSchema` is what `utoipa`'s derive uses for (generic) field types.
// It also provides `utoipa::PartialSchema` via a blanket implementation.
impl<P: FeatureProperties + ToSchema> utoipa::__dev::ComposeSchema for GeoJsonFeatureCollection<P> {
    fn compose(_generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        let geometry = P::GEOMETRY_TYPES
            .iter()
            .fold(AnyOfBuilder::new(), |any_of, geometry_type| {
                any_of.item(Ref::new(geometry_type.schema_url()))
            });

        let feature = ObjectBuilder::new()
            .property("geometry", geometry)
            .property("properties", P::schema());

        AllOfBuilder::new()
            .item(Ref::new(FEATURE_COLLECTION_SCHEMA_URL))
            .item(ObjectBuilder::new().property("features", ArrayBuilder::new().items(feature)))
            .into()
    }
}

impl<P: FeatureProperties + ToSchema> ToSchema for GeoJsonFeatureCollection<P> {
    fn name() -> Cow<'static, str> {
        format!("GeoJsonFeatureCollection_{}", P::name()).into()
    }

    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        P::schemas(schemas);
    }
}

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

        Ok(GeoJsonFeatureCollection::from(feature_collection))
    }
}

impl<P> AsRef<FeatureCollection> for GeoJsonFeatureCollection<P> {
    fn as_ref(&self) -> &FeatureCollection {
        &self.0
    }
}

impl<P> From<FeatureCollection> for GeoJsonFeatureCollection<P> {
    fn from(fc: FeatureCollection) -> Self {
        GeoJsonFeatureCollection(fc, PhantomData)
    }
}

/// A `GeoJSON` `FeatureCollection` input
// The features are described by `P` (cf. [`GeoJsonFeatureCollection`]).
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCollectionGeoJsonInput<P: FeatureProperties> {
    #[schema(inline)]
    pub value: GeoJsonFeatureCollection<P>,
    pub media_type: GeoJsonInputMediaType,
}

impl<P: FeatureProperties> FeatureCollectionGeoJsonInput<P> {
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
