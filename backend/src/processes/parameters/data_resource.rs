use crate::processes::util::json_input_value;
use ogcapi::types::processes::InputValue;
use serde::{Deserialize, Serialize};

/// Cf. <https://github.com/juhaku/utoipa/issues/1346>
#[derive(Debug)]
pub struct DataResourceSchema;

/// Data resources for outputting tabular data with JSON.
/// Based on <https://datapackage.org/profiles/2.0/dataresource.json>.
#[derive(Serialize, Debug)]
pub struct DataResource<R> {
    pub name: String,
    pub data: R,
    pub schema: Fields,
}

impl<R: Serialize> DataResource<R> {
    pub fn to_input_value(&self) -> anyhow::Result<InputValue> {
        Ok(json_input_value(serde_json::to_value(self)?))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Fields {
    pub fields: Vec<TableSchemaField>,
    pub primary_key: Option<Vec<String>>,
}

/// Field specification for Table Schema, based on <https://datapackage.org/standard/table-schema/>.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TableSchemaField {
    pub name: String,
    #[serde(default)]
    pub r#type: Option<TableSchemaType>,
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_type: Option<TableSchemaItemType>,
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
    List,
    // TODO: more types
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum TableSchemaItemType {
    #[default]
    String,
    Number,
    Integer,
    Boolean,
    // TODO: more types
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_serializes_and_deserializes_table_schema_fields() {
        let field = TableSchemaField {
            name: "test_field".to_string(),
            r#type: Some(TableSchemaType::Number),
            title: Some("Test Field".to_string()),
            item_type: Some(TableSchemaItemType::Integer),
        };

        let json = serde_json::to_value(&field).unwrap();
        let deserialized: TableSchemaField = serde_json::from_value(json).unwrap();

        assert_eq!(deserialized.name, "test_field");
        assert!(matches!(deserialized.r#type, Some(TableSchemaType::Number)));
        assert_eq!(deserialized.title, Some("Test Field".to_string()));
        assert!(matches!(
            deserialized.item_type,
            Some(TableSchemaItemType::Integer)
        ));
    }

    #[test]
    fn it_handles_fields_with_primary_key() {
        let fields = Fields {
            fields: vec![
                TableSchemaField {
                    name: "id".to_string(),
                    r#type: Some(TableSchemaType::Integer),
                    title: None,
                    item_type: None,
                },
                TableSchemaField {
                    name: "name".to_string(),
                    r#type: Some(TableSchemaType::String),
                    title: Some("Name".to_string()),
                    item_type: None,
                },
            ],
            primary_key: Some(vec!["id".to_string()]),
        };

        let json = serde_json::to_value(&fields).unwrap();
        let deserialized: Fields = serde_json::from_value(json).unwrap();

        assert_eq!(deserialized.fields.len(), 2);
        assert_eq!(deserialized.fields[0].name, "id");
        assert_eq!(deserialized.primary_key, Some(vec!["id".to_string()]));
    }

    #[test]
    fn it_converts_data_resource_to_input_value() {
        let data_resource = DataResource {
            name: "test_resource".to_string(),
            data: vec!["item1", "item2"],
            schema: Fields {
                fields: vec![TableSchemaField {
                    name: "value".to_string(),
                    r#type: Some(TableSchemaType::String),
                    title: None,
                    item_type: None,
                }],
                primary_key: None,
            },
        };

        let input_value = data_resource.to_input_value().unwrap();

        // Verify that the InputValue was created successfully
        let json_value = serde_json::to_value(&input_value).unwrap();
        assert!(json_value.is_object());
        let obj = json_value.as_object().unwrap();
        assert_eq!(
            obj.get("name").and_then(|v| v.as_str()),
            Some("test_resource")
        );
        assert!(obj.contains_key("data"));
        assert!(obj.contains_key("schema"));
    }

    #[test]
    fn it_handles_schema_types_and_item_types() {
        // Test all TableSchemaType variants serialize correctly
        let types = vec![
            (TableSchemaType::String, "string"),
            (TableSchemaType::Number, "number"),
            (TableSchemaType::Integer, "integer"),
            (TableSchemaType::Boolean, "boolean"),
            (TableSchemaType::List, "list"),
        ];

        for (schema_type, expected_str) in types {
            let json = serde_json::to_value(&schema_type).unwrap();
            assert_eq!(json.as_str(), Some(expected_str));
        }

        // Test all TableSchemaItemType variants serialize correctly
        let item_types = vec![
            (TableSchemaItemType::String, "string"),
            (TableSchemaItemType::Number, "number"),
            (TableSchemaItemType::Integer, "integer"),
            (TableSchemaItemType::Boolean, "boolean"),
        ];

        for (item_type, expected_str) in item_types {
            let json = serde_json::to_value(&item_type).unwrap();
            assert_eq!(json.as_str(), Some(expected_str));
        }
    }
}
