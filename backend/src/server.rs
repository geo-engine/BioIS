use crate::{
    auth::GeoEngineAuthMiddlewareLayer,
    collection_transactions::NoCollectionTransactions,
    config::CONFIG,
    db::setup_db,
    handler,
    jobs::JobHandler,
    processes::{HabitatDistanceProcess, NDVIProcess, ProcessesOpenApiSpec},
    state::spawn_with_user,
};
use ogcapi::{
    processes::{Processor, echo::Echo},
    services::{self as ogcapi_services},
};
use std::mem;
use utoipa::{
    Modify, OpenApi as _,
    openapi::{
        ContactBuilder, OpenApi, Ref,
        schema::{KnownFormat, ObjectBuilder, OneOfBuilder, Schema, SchemaFormat, Type},
    },
};
use utoipa_axum::{router::OpenApiRouter, routes};

/// Create and configure the OGC API service, including routes, state, and OpenAPI documentation.
pub async fn server() -> anyhow::Result<ogcapi_services::Service> {
    let db_pool = setup_db(&CONFIG.database).await?;

    let mut misc_router = OpenApiRouter::new()
        .routes(routes!(handler::health_handler))
        .nest("/auth", handler::auth_router())
        .with_state(CONFIG.geoengine.api_config(None));

    misc_router
        .get_openapi_mut()
        .merge(ProcessesOpenApiSpec::openapi());

    let mut processors: Vec<Box<dyn Processor>> = vec![Box::new(Echo), Box::new(NDVIProcess)];
    add_habitat_distance_process(&mut processors, db_pool.clone()).await;

    let drivers = ogcapi_services::Drivers {
        jobs: Box::new(JobHandler::new(db_pool).await?),
        collections: Box::new(NoCollectionTransactions),
    };

    let ogcapi_state = ogcapi_services::AppState::new(drivers)
        .await
        .processors(processors)
        .with_spawn_fn(spawn_with_user);

    let mut server_cfg: ogcapi_services::Config = (&CONFIG.server).into();
    if cfg!(test) {
        // Use ephemeral port in tests to avoid "address already in use" when creating the service
        server_cfg.port = 0;
    }

    let mut service = ogcapi_services::Service::try_new(&server_cfg, ogcapi_state)
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
    ResultsSchemaModifier.modify(openapi);

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

struct ResultsSchemaModifier;

impl Modify for ResultsSchemaModifier {
    fn modify(&self, openapi: &mut OpenApi) {
        let Some(components) = openapi.components.as_mut() else {
            tracing::warn!("OpenAPI components missing; skipping typed Results schema override");
            return;
        };

        let binary_schema = Schema::Object(
            ObjectBuilder::new()
                .schema_type(Type::String)
                .format(Some(SchemaFormat::KnownFormat(KnownFormat::Binary)))
                .build(),
        );

        let results_schema = OneOfBuilder::new()
            .item(Ref::from_schema_name("NDVIProcessOutputs"))
            .item(Ref::from_schema_name("HabitatDistanceProcessOutputs"))
            .item(binary_schema)
            .title(Some("ExecuteResults"));

        components
            .schemas
            .insert("Results".to_string(), results_schema.into());
    }
}

async fn add_habitat_distance_process(
    processors: &mut Vec<Box<dyn Processor>>,
    db_pool: crate::db::DbPool,
) {
    match HabitatDistanceProcess::new(db_pool).await {
        Ok(habitat_distance_process) => {
            tracing::info!(
                "Successfully initialized HabitatDistanceProcess, adding it to the list of available processes."
            );
            processors.push(Box::new(habitat_distance_process));
        }
        Err(err) => {
            tracing::warn!(
                "Failed to initialize HabitatDistanceProcess, skipping it: {:#}",
                err
            );
        }
    }
}
