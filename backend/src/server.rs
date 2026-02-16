use crate::{
    auth::GeoEngineAuthMiddlewareLayer,
    collection_transactions::NoCollectionTransactions,
    config::CONFIG,
    db::setup_db,
    handler,
    jobs::JobHandler,
    processes::{NDVIProcess, ProcessesOpenApiSpec},
    state::spawn_with_user,
};
use ogcapi::{
    processes::echo::Echo,
    services::{self as ogcapi_services},
};
use std::mem;
use utoipa::{
    OpenApi as _,
    openapi::{ContactBuilder, OpenApi},
};
use utoipa_axum::router::OpenApiRouter;

/// Create and configure the OGC API service, including routes, state, and OpenAPI documentation.
pub async fn server() -> anyhow::Result<ogcapi_services::Service> {
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
        .processors(vec![Box::new(Echo), Box::new(NDVIProcess)])
        .with_spawn_fn(spawn_with_user);

    let mut service = ogcapi_services::Service::try_new(&(&CONFIG.server).into(), ogcapi_state)
        .await?
        .processes_api();

    let router = service.get_router_mut();
    *router = mem::take(router)
        .merge(misc_router)
        .layer(GeoEngineAuthMiddlewareLayer);
    add_openapi_info(router.get_openapi_mut());

    Ok(service)
}

/// Modify the OpenAPI spec to add general information about the API, such as title, version, description, contact, etc.
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
