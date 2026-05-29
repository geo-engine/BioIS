use ogcapi::types::processes::InputValue;
// use serde::{Deserialize, Serialize, ser::Error as _};

/// Convert a [`serde_json::Value`] to an [`InputValue::Object`], returning an empty object if the value is not an object.
pub fn json_input_value(value: serde_json::Value) -> InputValue {
    InputValue::Object(match value {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    })
}
