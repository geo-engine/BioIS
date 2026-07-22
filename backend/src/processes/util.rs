use crate::processes::parameters::{BoundingBox, Year};
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use geoengine_api_client::models::{
    TimeDescriptor, TypedRasterResultDescriptor, TypedResultDescriptor, TypedVectorResultDescriptor,
};
use ogcapi::types::{
    common::Crs,
    processes::{InputValue, Output},
};
use std::collections::{HashMap, HashSet};

/// Convert a [`serde_json::Value`] to an [`InputValue::Object`], returning an empty object if the value is not an object.
pub fn json_input_value(value: serde_json::Value) -> InputValue {
    InputValue::Object(match value {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    })
}

/// Extract the keys of requested outputs from the provided `outputs` map.
/// If no outputs were requested, return a set containing all possible output keys.
pub fn to_output_keys(
    outputs: &HashMap<String, Output>,
    keys: impl IntoIterator<Item = &'static str>,
) -> Result<HashSet<String>> {
    let keys: HashSet<String> = keys.into_iter().map(ToString::to_string).collect();

    if outputs.is_empty() {
        return Ok(keys); // If no outputs were requested, default to all outputs
    }

    let output_keys: HashSet<String> = outputs.keys().cloned().collect();

    // Validate that all requested outputs are known
    for key in &output_keys {
        if !keys.contains(key) {
            return Err(anyhow::anyhow!("Unknown output key requested: {key}"));
        }
    }

    Ok(output_keys)
}

impl TryFrom<&TypedVectorResultDescriptor> for BoundingBox {
    type Error = anyhow::Error;

    fn try_from(value: &TypedVectorResultDescriptor) -> Result<Self, Self::Error> {
        let Some(bbox) = &value.bbox else {
            return Err(anyhow::anyhow!(
                "Expected bounding box in result descriptor, but it was None",
            ));
        };
        Ok(BoundingBox::new(
            bbox.lower_left_coordinate.x,
            bbox.lower_left_coordinate.y,
            bbox.upper_right_coordinate.x,
            bbox.upper_right_coordinate.y,
            spatial_reference_to_crs(&value.spatial_reference)?,
        ))
    }
}

pub fn vector_result_descriptor(
    typed_result_descriptor: TypedResultDescriptor,
) -> Result<TypedVectorResultDescriptor> {
    match typed_result_descriptor {
        TypedResultDescriptor::Vector(vector_result_descriptor) => Ok(*vector_result_descriptor),
        TypedResultDescriptor::Raster(_) => Err(anyhow::anyhow!(
            "Expected vector result descriptor, got raster"
        )),
        TypedResultDescriptor::Plot(_) => Err(anyhow::anyhow!(
            "Expected vector result descriptor, got plot"
        )),
    }
}

pub fn raster_result_descriptor(
    typed_result_descriptor: TypedResultDescriptor,
) -> Result<TypedRasterResultDescriptor> {
    match typed_result_descriptor {
        TypedResultDescriptor::Raster(raster_result_descriptor) => Ok(*raster_result_descriptor),
        TypedResultDescriptor::Vector(_) => Err(anyhow::anyhow!(
            "Expected raster result descriptor, got vector"
        )),
        TypedResultDescriptor::Plot(_) => Err(anyhow::anyhow!(
            "Expected raster result descriptor, got plot"
        )),
    }
}

fn spatial_reference_to_crs(spatial_reference: &str) -> Result<Crs> {
    if let Some(epsg_code) = spatial_reference.strip_prefix("EPSG:") {
        let epsg_code = epsg_code.parse::<i32>().map_err(|_| {
            anyhow::anyhow!("Invalid EPSG code in spatial reference: {spatial_reference}")
        })?;
        Ok(Crs::from_epsg(epsg_code))
    } else {
        Err(anyhow::anyhow!(
            "Unsupported spatial reference format: {spatial_reference}"
        ))
    }
}

/// Extracts the year range from a `TimeDescriptor`, returning an error if the time information is missing or invalid.
pub fn year_range_from_time_descriptor(time_descriptor: &TimeDescriptor) -> Result<(Year, Year)> {
    let time_interval = time_descriptor
        .bounds
        .as_ref()
        .and_then(Option::as_ref)
        .context("Missing time information")?;

    let start = DateTime::<Utc>::from_timestamp_millis(time_interval.start)
        .context("Invalid start time")?;
    let end =
        DateTime::<Utc>::from_timestamp_millis(time_interval.end).context("Invalid end time")?;

    Ok((Year(start.year() as u16), Year(end.year() as u16)))
}

pub fn set_min_max_in_schema(schema: &mut schemars::Schema, min: i64, max: i64) -> Result<()> {
    schema
        .as_object_mut()
        .context("Schema is not a number")?
        .insert("minimum".to_string(), serde_json::json!(min));
    schema
        .as_object_mut()
        .context("Schema is not a number")?
        .insert("maximum".to_string(), serde_json::json!(max));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_converts_json_values_to_input_values() {
        // Object value
        let obj_json = serde_json::json!({ "key": "value" });
        let input_value = json_input_value(obj_json);
        assert!(matches!(input_value, InputValue::Object(_)));

        // Non-object values (should return empty object)
        let non_obj_values = vec![
            serde_json::json!(null),
            serde_json::json!(true),
            serde_json::json!(42),
            serde_json::json!("string"),
            serde_json::json!([1, 2, 3]),
        ];

        for value in non_obj_values {
            let input_value = json_input_value(value);
            let InputValue::Object(map) = input_value else {
                panic!("Expected InputValue::Object");
            };
            assert!(map.is_empty());
        }
    }

    #[test]
    fn it_handles_output_keys_selection() {
        let keys = vec!["output1", "output2", "output3"];

        // No outputs requested - should return all keys
        let outputs: HashMap<String, Output> = HashMap::new();
        let result = to_output_keys(&outputs, keys.clone()).unwrap();
        assert_eq!(result.len(), 3);
        assert!(result.contains("output1"));
        assert!(result.contains("output2"));
        assert!(result.contains("output3"));
    }

    #[test]
    fn it_parses_spatial_references() {
        // Valid EPSG codes
        assert!(spatial_reference_to_crs("EPSG:4326").is_ok());
        assert!(spatial_reference_to_crs("EPSG:3857").is_ok());

        // Invalid EPSG code format
        assert!(spatial_reference_to_crs("EPSG:abc").is_err());
        assert!(spatial_reference_to_crs("EPSG:").is_err());

        // Unsupported formats
        assert!(spatial_reference_to_crs("OGC:CRS84").is_err());
        assert!(spatial_reference_to_crs("4326").is_err());
    }

    #[test]
    fn it_validates_output_keys() {
        let keys = vec!["output1", "output2", "output3"];

        // Selects requested output keys
        let mut outputs = HashMap::new();
        outputs.insert(
            "output1".to_string(),
            serde_json::from_value::<Output>(serde_json::json!({})).unwrap(),
        );
        outputs.insert(
            "output2".to_string(),
            serde_json::from_value::<Output>(serde_json::json!({})).unwrap(),
        );
        let result = to_output_keys(&outputs, keys.clone()).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains("output1") && result.contains("output2"));

        // Rejects unknown output keys
        let mut outputs = HashMap::new();
        outputs.insert(
            "unknown".to_string(),
            serde_json::from_value::<Output>(serde_json::json!({})).unwrap(),
        );
        let result = to_output_keys(&outputs, keys);
        assert!(result.is_err() && result.unwrap_err().to_string().contains("unknown"));
    }

    #[test]
    fn it_converts_vector_descriptor_to_bounding_box() {
        // Valid descriptor with bounding box
        let bbox = geoengine_api_client::models::BoundingBox2D {
            lower_left_coordinate: Box::new(geoengine_api_client::models::Coordinate2D {
                x: 0.0,
                y: 0.0,
            }),
            upper_right_coordinate: Box::new(geoengine_api_client::models::Coordinate2D {
                x: 10.0,
                y: 10.0,
            }),
        };
        let descriptor = TypedVectorResultDescriptor {
            bbox: Some(Box::new(bbox)),
            spatial_reference: "EPSG:4326".to_string(),
            ..Default::default()
        };
        assert!(BoundingBox::try_from(&descriptor).is_ok());

        // Missing bounding box
        let descriptor = TypedVectorResultDescriptor {
            bbox: None,
            spatial_reference: "EPSG:4326".to_string(),
            ..Default::default()
        };
        let result = BoundingBox::try_from(&descriptor);
        assert!(
            result.is_err()
                && result
                    .unwrap_err()
                    .to_string()
                    .contains("Expected bounding box")
        );
    }

    #[test]
    fn it_extracts_vector_descriptor() {
        // Extracts vector from typed result descriptor
        let vector_desc = TypedVectorResultDescriptor::default();
        let descriptor = TypedResultDescriptor::Vector(Box::new(vector_desc));
        assert!(vector_result_descriptor(descriptor).is_ok());

        // Rejects raster descriptor
        let result = vector_result_descriptor(TypedResultDescriptor::Raster(Box::default()));
        assert!(result.is_err() && result.unwrap_err().to_string().contains("Expected vector"));

        // Rejects plot descriptor
        let result = vector_result_descriptor(TypedResultDescriptor::Plot(Box::default()));
        assert!(result.is_err() && result.unwrap_err().to_string().contains("Expected vector"));
    }

    #[test]
    fn it_extracts_year_range_from_time_descriptor() {
        // Valid time descriptor
        let time_interval = TimeDescriptor {
            bounds: Some(Some(Box::new(geoengine_api_client::models::TimeInterval {
                start: 1_609_459_200_000, // 2021-01-01T00:00:00Z
                end: 1_640_995_200_000,   // 2022-01-01T00:00:00Z
            }))),
            ..Default::default()
        };
        let result = year_range_from_time_descriptor(&time_interval).unwrap();
        assert_eq!(result, (Year(2021), Year(2022)));

        // Missing time information
        let time_descriptor = TimeDescriptor {
            bounds: None,
            ..Default::default()
        };
        let result = year_range_from_time_descriptor(&time_descriptor);
        assert_eq!(result.unwrap_err().to_string(), "Missing time information");
    }

    #[test]
    fn it_sets_min_max_in_schema() {
        use schemars::generate::SchemaSettings;

        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;

        let mut generator = settings.into_generator();

        let mut schema = generator.root_schema_for::<Year>();
        let result = set_min_max_in_schema(&mut schema, 1, 10);
        assert!(result.is_ok());

        let schema_object = schema.as_object().expect("Schema should be an object");
        assert_eq!(schema_object.get("minimum").unwrap(), &serde_json::json!(1));
        assert_eq!(
            schema_object.get("maximum").unwrap(),
            &serde_json::json!(10)
        );
    }
}
