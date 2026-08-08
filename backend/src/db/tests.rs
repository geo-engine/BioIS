use crate::{
    CONFIG,
    db::{DbHandle, setup_db},
};
use std::{
    panic::AssertUnwindSafe,
    sync::{Arc, OnceLock},
};
use tokio::{
    runtime::Handle,
    sync::{OwnedSemaphorePermit, Semaphore},
};
use uuid::Uuid;

/// Execute a test function with a temporary database schema.
/// It will be cleaned up afterwards.
///
/// # Panics
///
/// Panics if the database setup fails or if the test function panics.
///
pub async fn with_temp_db<F, Fut, R>(f: F) -> R
where
    F: FnOnce(DbHandle) -> Fut + std::panic::UnwindSafe + Send + 'static,
    Fut: Future<Output = R>,
{
    let (permit, db_handle) = setup_test_db().await;

    // catch all panics and clean up first…
    let executed_fn = {
        let db_handle = db_handle.clone();
        std::panic::catch_unwind(
            AssertUnwindSafe /* fingers crossed for DbHandle */ (move || {
            tokio::task::block_in_place(move || {
                Handle::current().block_on(async move { f(db_handle).await })
            })
        }),
        )
    };

    tear_down_db(db_handle).await;
    drop(permit);

    match executed_fn {
        Ok(res) => res,
        Err(err) => std::panic::resume_unwind(err),
    }
}

/// configure the number of concurrently running tests that use the database
/// Keep at 1 to avoid PostgreSQL type OID caching issues when creating types
/// in parallel across different schemas. While each schema has its own isolated
/// types (for testing isolation), parallel type creation causes operator resolution
/// failures when Toasty generates SQL across different test schema OIDs.
const CONCURRENT_DB_TESTS: usize = 1;
static DB_SYNC: OnceLock<Arc<Semaphore>> = OnceLock::new();

/// Setup database schema and return its name.
pub(crate) async fn setup_test_db() -> (OwnedSemaphorePermit, DbHandle) {
    // acquire a permit from the semaphore that limits the number of concurrently running tests that use the database
    let permit = DB_SYNC
        .get_or_init(|| Arc::new(Semaphore::new(CONCURRENT_DB_TESTS)))
        .clone()
        .acquire_owned()
        .await
        .unwrap();

    let mut db_config = CONFIG.database.clone();
    db_config.schema = test_schema();
    let db_pool = setup_db(&db_config).await.unwrap();

    (permit, db_pool)
}

/// Tear down database schema.
pub(crate) async fn tear_down_db(mut db_pool: DbHandle) {
    let schema = &db_pool.schema_name();
    toasty::sql::statement(format!("DROP SCHEMA IF EXISTS {schema} CASCADE;"))
        .exec(db_pool.as_mut())
        .await
        .unwrap();
}

/// Generate random temp schema name for testing
fn test_schema() -> String {
    format!("geoengine_test_{}", Uuid::now_v7().as_simple())
}

#[test]
fn it_generates_test_schema_names() {
    let test_schema_name = test_schema();
    assert!(test_schema_name.starts_with("geoengine_test_"));
    assert!(
        test_schema_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "{test_schema_name} contains invalid characters"
    );
}
