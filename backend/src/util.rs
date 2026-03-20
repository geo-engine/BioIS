use anyhow::{Context, Result};
use geoengine_openapi_client::models::{
    LegacyTypedOperator, LegacyTypedOperatorOperator, VectorOperator, Workflow,
    legacy_typed_operator::Type as WorkflowType,
};
use std::ops::Deref;
use tracing::error;

/// Converts a Geo Engine operator to an Geo Engine OpenAPI workflow.
pub fn to_api_workflow(
    operator: &VectorOperator,
) -> Result<geoengine_openapi_client::models::Workflow> {
    let serde_json::Value::Object(mut json_object) = serde_json::to_value(operator)? else {
        anyhow::bail!("expected operator to serialize `TypedOperator` to a JSON object");
    };
    let serde_json::Value::String(r#type) =
        json_object.remove("type").context("missing `type` field")?
    else {
        anyhow::bail!("`type` field is not a string");
    };

    Ok(Workflow::LegacyTypedOperator(
        LegacyTypedOperator {
            operator: LegacyTypedOperatorOperator {
                params: json_object.remove("params"),
                sources: json_object.remove("sources"),
                r#type,
            }
            .into(),
            r#type: WorkflowType::Vector,
        }
        .into(),
    ))
}

pub fn error_response<T>(
    error: &geoengine_openapi_client::apis::Error<T>,
) -> Option<geoengine_openapi_client::models::ErrorResponse> {
    use geoengine_openapi_client::apis::Error as ApiError;
    use geoengine_openapi_client::models::ErrorResponse as ApiErrorResponse;

    match error {
        ApiError::Reqwest(_) | ApiError::Serde(_) | ApiError::Io(_) => None,
        ApiError::ResponseError(error) => {
            serde_json::from_str::<ApiErrorResponse>(&error.content).ok()
        }
    }
}

/// Helper function to read-lock a `RwLock`, recovering from poisoning if necessary
#[allow(unused)] // TODO: use or delete
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
#[allow(unused)] // TODO: use or delete
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
    use geoengine_openapi_client::models::{
        ColumnNames, Default as ColumnNamesDefault, FeatureAggregationMethod, GdalSource,
        GdalSourceParameters, RasterOperator, RasterVectorJoin, RasterVectorJoinParameters,
        SingleVectorMultipleRasterSources, TemporalAggregationMethod, VectorOperator,
    };
    use pretty_assertions::assert_eq;
    use std::sync::{Arc, RwLock};

    #[test]
    fn it_converts_operator_to_api_workflow() {
        let raster_vector_join = VectorOperator::RasterVectorJoin(
            RasterVectorJoin {
                r#type: Default::default(),
                params: RasterVectorJoinParameters {
                    feature_aggregation: FeatureAggregationMethod::First,
                    feature_aggregation_ignore_no_data: Some(true),
                    names: ColumnNames::Default(ColumnNamesDefault::new(Default::default()).into())
                        .into(),
                    temporal_aggregation: TemporalAggregationMethod::First,
                    temporal_aggregation_ignore_no_data: Some(true),
                }
                .into(),
                sources: SingleVectorMultipleRasterSources {
                    vector: VectorOperator::MockPointSource(Default::default()).into(),
                    rasters: vec![RasterOperator::GdalSource(
                        GdalSource {
                            r#type: Default::default(),
                            params: GdalSourceParameters {
                                data: "ndvi".into(),
                                overview_level: None,
                            }
                            .into(),
                        }
                        .into(),
                    )],
                }
                .into(),
            }
            .into(),
        );

        let api_workflow = to_api_workflow(&raster_vector_join).unwrap();

        assert_eq!(
            serde_json::to_string_pretty(&api_workflow).unwrap(),
            serde_json::to_string_pretty(&serde_json::json!({
                "type": "Vector",
                "operator": {
                    "type": "RasterVectorJoin",
                    "params": {
                        "featureAggregation": "first",
                        "featureAggregationIgnoreNoData": true,
                        "names": {
                            "type": "default"
                        },
                        "temporalAggregation": "first",
                        "temporalAggregationIgnoreNoData": true
                    },
                    "sources": {
                        "rasters": [{
                            "type": "GdalSource",
                            "params": {
                                "data": "ndvi"
                            }
                        }],
                        "vector": {
                            "type": "MockPointSource",
                            "params": {
                                "points": [],
                                "spatialBounds": {
                                    "type": "derive"
                                }
                            }
                        }
                    }
                }
            }))
            .unwrap()
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
