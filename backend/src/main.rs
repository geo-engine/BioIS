use std::mem;

use crate::{
    auth::GeoEngineAuthMiddlewareLayer, collection_transactions::NoCollectionTransactions,
    db::setup_db, jobs::JobHandler, processes::ProcessesOpenApiSpec,
};
use config::CONFIG;
use ogcapi::services as ogcapi_services;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    OpenApi as _,
    openapi::{ContactBuilder, OpenApi},
};
use utoipa_axum::router::OpenApiRouter;

mod auth;
mod collection_transactions;
mod config;
mod db;
mod handler;
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
    };

    let db_pool = setup_db(&CONFIG.database).await?;

    let mut misc_router = OpenApiRouter::new()
        .routes(handler::routes())
        .with_state(CONFIG.geoengine.api_config(None));

    misc_router
        .get_openapi_mut()
        .merge(ProcessesOpenApiSpec::openapi());

    let drivers = ogcapi_services::Drivers {
        jobs: Box::new(JobHandler::new(db_pool).await?),
        collections: Box::new(NoCollectionTransactions),
    };

    let ogcapi_state = ogcapi_services::AppState::new(drivers)
        .await
        .processors(vec![
            Box::new(ogcapi::processes::echo::Echo),
            Box::new(processes::NDVIProcess),
        ])
        .with_spawn_fn(state::spawn_with_user);

    let mut service = ogcapi_services::Service::try_new(&ogcapi_config, ogcapi_state)
        .await?
        .processes_api();

    let router = service.get_router_mut();
    *router = mem::take(router)
        .merge(misc_router)
        .layer(GeoEngineAuthMiddlewareLayer);
    add_openapi_info(router.get_openapi_mut());

    service.serve().await
}

fn add_openapi_info(openapi: &mut OpenApi) {
    openapi.info = utoipa::openapi::InfoBuilder::new()
        .title("BioIS API")
        .version(env!("CARGO_PKG_VERSION"))
        .description(Some("API for the BioIS service, providing access to geospatial processing and job management."))
        .contact(Some(
            ContactBuilder::new()
                .name(Some("Geo Engine GmbH"))
                .url(Some("https://www.geoengine.de/en/"))
                .email(Some("info@geoengine.de"))
                .build()
        ))
        // .terms_of_service(None) // TODO: add link to ToS
        .license(None) // TODO: add link to license
        .build();
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(CONFIG.logging.clone().into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}
