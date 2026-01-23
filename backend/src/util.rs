use std::ops::Deref;

use anyhow::{Context, Result};
use tracing::error;

/// Converts a Geo Engine operator to an Geo Engine OpenAPI workflow.
pub fn to_api_workflow(
    operator: &geoengine_operators::engine::TypedOperator,
) -> Result<geoengine_openapi_client::models::Workflow> {
    use geoengine_openapi_client::models::{
        TypedOperatorOperator, Workflow, workflow::Type as WorkflowType,
    };
    use geoengine_operators::engine::TypedOperator;

    let serde_json::Value::Object(mut json_object) = serde_json::to_value(operator)? else {
        anyhow::bail!("expected operator to serialize `TypedOperator` to a JSON object");
    };
    let serde_json::Value::Object(mut json_object) = json_object
        .remove("operator")
        .context("missing `operator` field")?
    else {
        anyhow::bail!("`operator` field is not a JSON object");
    };
    let serde_json::Value::String(r#type) =
        json_object.remove("type").context("missing `type` field")?
    else {
        anyhow::bail!("`type` field is not a string");
    };

    Ok(Workflow {
        operator: Box::new(TypedOperatorOperator {
            params: json_object.remove("params"),
            sources: json_object.remove("sources"),
            r#type,
        }),
        r#type: match operator {
            TypedOperator::Vector(..) => WorkflowType::Vector,
            TypedOperator::Raster(..) => WorkflowType::Raster,
            TypedOperator::Plot(..) => WorkflowType::Plot,
        },
    })
}

/// Helper function to read-lock a `RwLock`, recovering from poisoning if necessary.
pub(crate) fn read_lock<T>(mutex: &std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match mutex.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Mutex was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

/// Helper function to write-lock a `RwLock`, recovering from poisoning if necessary.
pub(crate) fn write_lock<T>(mutex: &std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match mutex.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Mutex was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

/// A wrapper type to hide sensitive information in Debug implementations.
pub struct Secret<T>(pub T);

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "********")
    }
}

impl<T> std::fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "********")
    }
}

impl<T> Deref for Secret<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> Clone for Secret<T> {
    fn clone(&self) -> Self {
        Secret(self.0.clone())
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Secret(value)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::{Arc, RwLock};

    #[test]
    fn it_converts_operator_to_api_workflow() {
        use geoengine_datatypes::{dataset::NamedData, raster::RasterDataType};
        use geoengine_operators::engine::{RasterOperator, TypedOperator};
        use geoengine_operators::processing::{RasterTypeConversion, RasterTypeConversionParams};
        use geoengine_operators::source::{GdalSource, GdalSourceParameters};

        let gdal_source = TypedOperator::Raster(
            RasterTypeConversion {
                params: RasterTypeConversionParams {
                    output_data_type: RasterDataType::F32,
                },
                sources: GdalSource {
                    params: GdalSourceParameters {
                        data: NamedData::with_system_name("ndvi"),
                    },
                }
                .boxed()
                .into(),
            }
            .boxed(),
        );

        let api_workflow = to_api_workflow(&gdal_source).unwrap();

        assert_eq!(
            serde_json::to_value(api_workflow).unwrap(),
            serde_json::json!({
                "type": "Raster",
                "operator": {
                    "type": "RasterTypeConversion",
                    "params": {
                        "outputDataType": "F32"
                    },
                    "sources": {
                        "raster": {
                            "type": "GdalSource",
                            "params": {
                                "data": "ndvi"
                            }
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn it_hides_secret_in_debug_and_display() {
        let secret = Secret("my_password".to_string());
        assert_eq!(format!("{:?}", secret), "********");
        assert_eq!(format!("{}", secret), "********");
    }

    #[test]
    fn it_recovers_from_poisoned_read_lock() {
        let lock = Arc::new(RwLock::new(42));

        // Poison the lock by panicking while holding a write lock
        {
            let lock = Arc::clone(&lock);
            let _ = std::thread::spawn(move || {
                let _guard = lock.write().unwrap();
                panic!("poison!");
            })
            .join();
        }

        // Should recover and read the value
        let value = *read_lock(&lock);
        assert_eq!(value, 42);
    }

    #[test]
    fn it_recovers_from_poisoned_write_lock() {
        let lock = Arc::new(RwLock::new(100));

        // Poison the lock by panicking while holding a write lock
        {
            let lock = Arc::clone(&lock);
            let _ = std::thread::spawn(move || {
                let _guard = lock.write().unwrap();
                panic!("poison!");
            })
            .join();
        }

        // Should recover and allow writing
        {
            let mut guard = write_lock(&lock);
            *guard = 200;
        }
        assert_eq!(*read_lock(&lock), 200);
    }

    #[test]
    fn it_reads_and_writes_with_unpoisoned_lock() {
        let lock = RwLock::new(5);

        {
            let guard = read_lock(&lock);
            assert_eq!(*guard, 5);
        }

        {
            let mut guard = write_lock(&lock);
            *guard = 10;
        }

        {
            let guard = read_lock(&lock);
            assert_eq!(*guard, 10);
        }
    }
}
