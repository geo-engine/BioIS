use crate::db::setup_db;
use crate::state::{AppState, BoxedProcessor};
use config::CONFIG;
use ogcapi::{processes as ogcapi_processes, services as ogcapi_services};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod db;
mod jobs;
mod processes;
mod state;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let ogcapi_config = ogcapi::services::Config {
        host: CONFIG.server.host.clone(),
        port: CONFIG.server.port,
        openapi: None,
    };

    let db_pool = setup_db(&CONFIG.database)?;

    let ogcapi_state = AppState::new(db_pool).with_processors([
        Box::new(ogcapi_processes::echo::Echo::default()),
        Box::new(processes::NDVIProcess),
    ] as [BoxedProcessor; _]);

    // Build & run with hyper
    let ogcapi_service = ogcapi_services::Service::try_new_with(&ogcapi_config, ogcapi_state)
        .await?
        .with_processes();

    ogcapi_service.serve().await;

    Ok(())
}

fn setup_tracing() {
    // setup tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}
