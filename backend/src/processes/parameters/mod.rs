use geoengine_api_client::models::{BoundingBox2D, Coordinate2D, ProvenanceEntry};
use geojson::Position;
use ogcapi::types::{
    common::Crs,
    processes::description::{DescriptionType, InputDescription, Metadata, OutputDescription},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

pub use data_resource::{
    DataResource, DataResourceSchema, Fields, TableSchemaField, TableSchemaItemType,
    TableSchemaType,
};
#[cfg(test)]
pub use geo_json::GeoJsonInputMediaType;
pub use geo_json::{
    FeatureCollectionGeoJsonInput, GeoJsonFeatureCollection, PointGeoJson, PointGeoJsonInput,
    geojson_feature_collection_utils, geojson_feature_utils,
};
#[cfg(test)]
pub use units::Hectare;
pub use units::{Area, Kilometers, Month, SquareMeter, UnitForArea, Year};

mod data_resource;
mod geo_json;
mod units;

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
    "https://datapackage.org/profiles/2.0/dataresource.json",
    R
);

impl_extern_schema!(
    DataResourceSchema,
    "Data Resource",
    "https://datapackage.org/profiles/2.0/dataresource.json",
);

/// Documentation source for audit and provenance, e.g. a Geo Engine workflow or a scientific paper.
/// This is included in the outputs of the process for traceability and auditing purposes.
#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct DocumentationSource {
    /// A human-readable identifier of the documentation source (e.g. "Geo Engine workflow XYZ")
    pub data: String,
    /// A description, citation or URL pointing to the source of the documentation (e.g. a link to a Geo Engine workflow, or a scientific paper)
    pub documentation_source: String,
}

impl DocumentationSource {
    const DATA_FIELD_NAME: &'static str = "data";
    const DOCUMENTATION_SOURCE_FIELD_NAME: &'static str = "documentation_source";
}

impl From<ProvenanceEntry> for DocumentationSource {
    fn from(entry: ProvenanceEntry) -> Self {
        DocumentationSource {
            data: entry.provenance.citation.clone(),
            documentation_source: format!(
                "URI: <a href=\"{}\">{}</a>\nLicense: {}",
                entry.provenance.uri, entry.provenance.uri, entry.provenance.license
            ),
        }
    }
}

impl From<Vec<DocumentationSource>> for DataResource<Vec<DocumentationSource>> {
    fn from(value: Vec<DocumentationSource>) -> Self {
        Self {
            name: "Documentation Sources".to_string(),
            data: value,
            schema: Fields {
                fields: vec![
                    TableSchemaField {
                        name: DocumentationSource::DATA_FIELD_NAME.to_string(),
                        r#type: Some(TableSchemaType::String),
                        title: Some("Data".to_string()),
                        ..Default::default()
                    },
                    TableSchemaField {
                        name: DocumentationSource::DOCUMENTATION_SOURCE_FIELD_NAME.to_string(),
                        r#type: Some(TableSchemaType::String),
                        title: Some("Documentation Source".to_string()),
                        ..Default::default()
                    },
                ],
                primary_key: vec![DocumentationSource::DATA_FIELD_NAME.to_string()].into(),
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

/// Helper struct to define input specifications ([`InputDescription`]) for processes, including key, title, description, and type.
pub struct InputSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub metadata: Vec<Metadata>,
    pub r#type: schemars::Schema,
}

/// Helper trait to convert an array of input specifications into a [`HashMap`] of [`InputDescription`] for processes.
pub trait ToInputHashMap {
    fn into_hash_map(self) -> HashMap<String, InputDescription>;
}

impl<const N: usize> ToInputHashMap for [InputSpec; N] {
    fn into_hash_map(self) -> HashMap<String, InputDescription> {
        self.into_iter()
            .map(move |input| {
                (
                    input.key.to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: input.title.to_string().into(),
                            description: input.description.to_string().into(),
                            metadata: input.metadata,
                            ..Default::default()
                        },
                        schema: input.r#type.to_value(),
                        ..Default::default()
                    },
                )
            })
            .collect()
    }
}

/// Helper struct to define output specifications ([`OutputDescription`]) for processes, including key, title, description, and type.
pub struct OutputSpec {
    pub key: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub r#type: schemars::Schema,
}

/// Helper trait to convert an array of output specifications into a [`HashMap`] of [`OutputDescription`] for processes.
pub trait ToOutputHashMap {
    fn into_hash_map(self) -> HashMap<String, OutputDescription>;
}

impl<const N: usize> ToOutputHashMap for [OutputSpec; N] {
    fn into_hash_map(self) -> HashMap<String, OutputDescription> {
        self.into_iter()
            .map(move |output| {
                (
                    output.key.to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: output.title.to_string().into(),
                            description: output.description.to_string().into(),
                            ..Default::default()
                        },
                        schema: output.r#type.to_value(),
                    },
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    minx: f64,
    miny: f64,
    maxx: f64,
    maxy: f64,
    crs: Crs,
}

impl BoundingBox {
    pub fn new(minx: f64, miny: f64, maxx: f64, maxy: f64, crs: Crs) -> Self {
        Self {
            minx,
            miny,
            maxx,
            maxy,
            crs,
        }
    }

    pub fn new_invalid(crs: Crs) -> Self {
        Self {
            minx: f64::MAX,
            miny: f64::MAX,
            maxx: f64::MIN,
            maxy: f64::MIN,
            crs,
        }
    }

    pub fn enlarge_by_positions<'p>(&mut self, other: impl Iterator<Item = &'p Position>) {
        for position in other {
            self.minx = self.minx.min(position[0]);
            self.miny = self.miny.min(position[1]);
            self.maxx = self.maxx.max(position[0]);
            self.maxy = self.maxy.max(position[1]);
        }
    }

    pub fn wfs_string(&self) -> String {
        format!(
            "{minx},{miny},{maxx},{maxy}",
            minx = self.minx,
            miny = self.miny,
            maxx = self.maxx,
            maxy = self.maxy
        )
    }

    pub fn crs(&self) -> &Crs {
        &self.crs
    }

    pub fn to_bounding_box_2d(&self) -> BoundingBox2D {
        BoundingBox2D {
            lower_left_coordinate: Box::new(Coordinate2D {
                x: self.minx,
                y: self.miny,
            }),
            upper_right_coordinate: Box::new(Coordinate2D {
                x: self.maxx,
                y: self.maxy,
            }),
        }
    }
}

/// Helper struct to define complex input specifications for processes.
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonInput<T> {
    #[schema(inline)]
    pub value: T,
    pub media_type: JsonInputMediaType,
}

impl<T> Default for JsonInput<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            value: T::default(),
            media_type: JsonInputMediaType::Json,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum JsonInputMediaType {
    #[serde(rename = "application/json")]
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_converts_documentation_sources_into_data_resource() {
        let sources = vec![DocumentationSource {
            data: "Geo Engine workflow XYZ".to_string(),
            documentation_source: "https://example.com/workflow/xyz".to_string(),
        }];

        let data_resource: DataResource<Vec<DocumentationSource>> = sources.into();

        assert_eq!(data_resource.data.len(), 1);
        assert_eq!(data_resource.data[0].data, "Geo Engine workflow XYZ");
        assert_eq!(
            data_resource.data[0].documentation_source,
            "https://example.com/workflow/xyz"
        );

        assert_eq!(data_resource.schema.fields.len(), 2);
        assert_eq!(data_resource.schema.fields[0].name, "data");
        assert!(matches!(
            data_resource.schema.fields[0].r#type,
            Some(TableSchemaType::String)
        ));
        assert_eq!(
            data_resource.schema.fields[0].title,
            Some("Data".to_string())
        );

        assert_eq!(data_resource.schema.fields[1].name, "documentation_source");
        assert!(matches!(
            data_resource.schema.fields[1].r#type,
            Some(TableSchemaType::String)
        ));
        assert_eq!(
            data_resource.schema.fields[1].title,
            Some("Documentation Source".to_string())
        );
    }

    #[test]
    fn it_creates_and_manipulates_bounding_boxes() {
        let crs = Crs::default2d();

        // Test new()
        let bbox = BoundingBox::new(1.0, 2.0, 3.0, 4.0, crs.clone());
        assert_eq!(bbox.wfs_string(), "1,2,3,4");
        assert_eq!(bbox.crs(), &crs);

        // Test new_invalid()
        let invalid_bbox = BoundingBox::new_invalid(crs.clone());
        assert_eq!(
            invalid_bbox.wfs_string(),
            format!("{},{},{},{}", f64::MAX, f64::MAX, f64::MIN, f64::MIN)
        );

        // Test to_bounding_box_2d()
        let bbox_2d = bbox.to_bounding_box_2d();
        assert_abs_diff_eq!(bbox_2d.lower_left_coordinate.x, 1.0);
        assert_abs_diff_eq!(bbox_2d.lower_left_coordinate.y, 2.0);
        assert_abs_diff_eq!(bbox_2d.upper_right_coordinate.x, 3.0);
        assert_abs_diff_eq!(bbox_2d.upper_right_coordinate.y, 4.0);
    }

    #[test]
    fn it_enlarges_bounding_box_with_positions() {
        let crs = Crs::default2d();
        let mut bbox = BoundingBox::new(10.0, 20.0, 30.0, 40.0, crs);

        let positions = [
            geojson::Position::from(vec![5.0, 15.0]),
            geojson::Position::from(vec![35.0, 45.0]),
        ];

        bbox.enlarge_by_positions(positions.iter());
        assert_eq!(bbox.wfs_string(), "5,15,35,45");
    }

    #[test]
    fn it_generates_correct_schema_for_json_input_with_generic_type() {
        #[derive(Serialize, Deserialize, JsonSchema, ToSchema)]
        #[serde(rename_all = "camelCase")]
        struct TestPayload {
            text: String,
            count: f64,
        }

        let root_schema = schemars::schema_for!(JsonInput<TestPayload>);
        let schema_json = serde_json::to_value(&root_schema).expect("Failed to serialize schema");

        let expected = serde_json::json!({
            "$defs": {
                "JsonInputMediaType": {
                    "enum": ["application/json"],
                    "type": "string"
                },
                "TestPayload": {
                    "properties": {
                        "count": {
                            "format": "double",
                            "type": "number"
                        },
                        "text": {
                            "type": "string"
                        }
                    },
                    "required": ["text", "count"],
                    "type": "object"
                }
            },
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "description": "Helper struct to define complex input specifications for processes.",
            "properties": {
                "mediaType": {
                    "$ref": "#/$defs/JsonInputMediaType"
                },
                "value": {
                    "$ref": "#/$defs/TestPayload"
                }
            },
            "required": ["value", "mediaType"],
            "title": "JsonInput",
            "type": "object"
        });

        assert_eq!(schema_json, expected);

        let option_root_schema = schemars::schema_for!(Option<JsonInput<TestPayload>>);
        let option_schema_json =
            serde_json::to_value(&option_root_schema).expect("Failed to serialize schema");

        let option_expected = serde_json::json!({
            "$defs": {
                "JsonInput": {
                    "description": "Helper struct to define complex input specifications for processes.",
                    "properties": {
                        "mediaType": {
                            "$ref": "#/$defs/JsonInputMediaType"
                        },
                        "value": {
                            "$ref": "#/$defs/TestPayload"
                        }
                    },
                    "required": ["value", "mediaType"],
                    "type": "object"
                },
                "JsonInputMediaType": {
                    "enum": ["application/json"],
                    "type": "string"
                },
                "TestPayload": {
                    "properties": {
                        "count": {
                            "format": "double",
                            "type": "number"
                        },
                        "text": {
                            "type": "string"
                        }
                    },
                    "required": ["text", "count"],
                    "type": "object"
                }
            },
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "anyOf": [
                {
                    "$ref": "#/$defs/JsonInput"
                },
                {
                    "type": "null"
                }
            ],
            "title": "Nullable_JsonInput"
        });

        assert_eq!(option_schema_json, option_expected);
    }
}
