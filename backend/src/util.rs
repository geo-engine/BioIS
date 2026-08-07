use geoengine_api_client::models::{
    RasterOperator, TypedOperator, TypedRasterOperator, TypedVectorOperator, VectorOperator,
    Workflow, typed_raster_operator::Type as RasterType, typed_vector_operator::Type as VectorType,
};
use serde::Deserialize;
use std::ops::Deref;
use tokio::task::JoinHandle;
use tracing::error;
use tracing_subscriber::{
    EnvFilter, filter::Directive, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Converts a Geo Engine operator to an Geo Engine OpenAPI workflow.
pub fn to_api_vector_process(operator: &VectorOperator) -> geoengine_api_client::models::Workflow {
    Workflow::TypedOperator(Box::new(TypedOperator::TypedVectorOperator(Box::new(
        TypedVectorOperator {
            operator: Box::new(operator.clone()),
            r#type: VectorType::Vector,
        },
    ))))
}

/// Converts a Geo Engine operator to an Geo Engine OpenAPI workflow.
pub fn to_api_raster_process(operator: &RasterOperator) -> geoengine_api_client::models::Workflow {
    Workflow::TypedOperator(Box::new(TypedOperator::TypedRasterOperator(Box::new(
        TypedRasterOperator {
            operator: Box::new(operator.clone()),
            r#type: RasterType::Raster,
        },
    ))))
}

pub fn error_response<T>(
    error: &geoengine_api_client::apis::Error<T>,
) -> Option<geoengine_api_client::models::ErrorResponse> {
    use geoengine_api_client::apis::Error as ApiError;
    use geoengine_api_client::models::ErrorResponse as ApiErrorResponse;

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

impl Secret<String> {
    /// Returns the inner string.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

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

impl<'de> Deserialize<'de> for Secret<String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Secret(s))
    }
}

/// Extracts the heading from a markdown string, if it starts with a level 1 heading (`# `).
pub fn md_heading(s: &str) -> &str {
    let mut lines = s.lines();
    let first_line = lines.next().unwrap_or("");
    if !first_line.starts_with("# ") {
        return "";
    }
    first_line.trim_start_matches("# ").trim()
}

/// Extracts the content from a markdown string, removing the first heading if it exists.
pub fn md_content(s: &str) -> &str {
    let mut lines = s.lines();
    let first_line = lines.next().unwrap_or("");
    if !first_line.starts_with("# ") {
        return s.trim();
    }

    // TODO: use `.remainder()` once it is stabilized

    let heading_end_index = s.find('\n').unwrap_or(0);

    let first_content_index = s[heading_end_index..]
        .find(|c: char| !c.is_whitespace())
        .map_or(heading_end_index, |idx| heading_end_index + idx);

    s[first_content_index..].trim()
}

pub fn setup_tracing(log_level: Directive) {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(log_level)
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}

#[cfg(test)]
#[allow(dead_code, reason = "Used in tests to setup tracing")]
pub fn setup_tracing_for_tests() {
    setup_tracing("info".parse().unwrap());
}

/// A macro to concatenate two static strings (`&'static str`) at compile time.
///
/// Do not confuse this with the `concat!` macro, which concatenates string literals at compile time.
/// This macro can concatenate any static strings, including those that are not literals.
///
/// # Panics
///
/// If the concatenated result is not valid UTF-8, this macro will panic at compile time.
#[macro_export]
macro_rules! const_concat {
    ($a:expr, $b:expr) => {{
        const A: &'static str = $a;
        const B: &'static str = $b;
        const LEN: usize = A.len() + B.len();

        const BYTES: [u8; LEN] = {
            let mut bytes = [0u8; LEN];
            let mut i = 0;
            while i < A.len() {
                bytes[i] = A.as_bytes()[i];
                i += 1;
            }
            let mut j = 0;
            while j < B.len() {
                bytes[A.len() + j] = B.as_bytes()[j];
                j += 1;
            }
            bytes
        };

        const RESULT: &'static str = match std::str::from_utf8(&BYTES) {
            Ok(s) => s,
            Err(_) => panic!("Invalid UTF-8"),
        };
        RESULT
    }};
}

/// A wrapper around `tokio::task::spawn_blocking` that keeps the current
/// `tracing` span active inside the blocking task.
#[inline]
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let parent_span = tracing::Span::current();

    tokio::task::spawn_blocking(move || {
        let _entered_span = parent_span.enter();

        f()
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use geoengine_api_client::models::{
        ColumnNames, Default as ColumnNamesDefault, FeatureAggregationMethod, GdalSource,
        GdalSourceParameters, RasterOperator, RasterVectorJoin, RasterVectorJoinParameters,
        SingleVectorMultipleRasterSources, TemporalAggregationMethod, VectorOperator,
    };
    use indoc::indoc;
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

        let api_workflow = to_api_vector_process(&raster_vector_join);

        assert_eq!(
            serde_json::to_value(&api_workflow).unwrap(),
            serde_json::json!({
                "type": "Vector",
                "operator": {
                    "type": "RasterVectorJoin",
                    "params": {
                        "names": {
                            "type": "default"
                        },
                        "featureAggregation": "first",
                        "featureAggregationIgnoreNoData": true,
                        "temporalAggregation": "first",
                        "temporalAggregationIgnoreNoData": true
                    },
                    "sources": {
                        "vector": {
                            "type": "MockPointSource",
                            "params": {
                                "points": [],
                                "spatialBounds": {
                                    "type": "derive"
                                }
                            },
                        },
                        "rasters": [{
                            "type": "GdalSource",
                            "params": {
                                "data": "ndvi"
                            },
                        }]
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

    #[test]
    fn it_extracts_md_content() {
        let md = indoc! {"
            # Heading

            Content
        "};
        assert_eq!(md_heading(md), "Heading");
        assert_eq!(md_content(md), "Content");

        let md_no_heading = indoc! {"
            Content without heading
        "};
        assert_eq!(md_heading(md_no_heading), "");
        assert_eq!(md_content(md_no_heading), "Content without heading");

        let md_only_heading = indoc! {"
            # Only Heading
        "};
        assert_eq!(md_heading(md_only_heading), "Only Heading");
        assert_eq!(md_content(md_only_heading), "");

        let md_heading_with_whitespace = indoc! {"
            # Heading with whitespace

            Content with leading and trailing whitespace
        "};
        assert_eq!(
            md_heading(md_heading_with_whitespace),
            "Heading with whitespace"
        );
        assert_eq!(
            md_content(md_heading_with_whitespace),
            "Content with leading and trailing whitespace"
        );
    }

    #[test]
    #[allow(clippy::items_after_statements, reason = "Makes sense for the test")]
    fn it_concats_const_strings() {
        assert_eq!(const_concat!("Hello, ", "World!"), "Hello, World!");
        assert_eq!(const_concat!("Foo", ""), "Foo");
        const BAR: &str = "Bar";
        assert_eq!(const_concat!("", BAR), "Bar");
    }

    #[tokio::test]
    async fn it_sets_up_tracing_and_spawns_blocking_task_with_preserved_span() {
        fn current_span_name() -> String {
            tracing::Span::current()
                .metadata()
                .map(|m| m.name().to_string())
                .unwrap_or_default()
        }

        let result = std::panic::catch_unwind(|| {
            setup_tracing("info".parse().unwrap());
            tracing::info!("tracing initialized");
        });

        assert!(result.is_ok());

        // Initialize a subscriber for this test
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();

        let span = tracing::info_span!("test_span");
        let _entered = span.enter();

        assert_eq!(
            spawn_blocking(current_span_name).await.unwrap(),
            "test_span"
        );

        // Test with tokio's spawn_blocking directly - span is lost
        assert_eq!(
            tokio::task::spawn_blocking(current_span_name)
                .await
                .unwrap(),
            ""
        );
    }
}
