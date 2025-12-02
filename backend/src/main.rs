use axum::routing::get;
use config::CONFIG;
use ogcapi::{processes as ogcapi_processes, services as ogcapi_services};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_axum::router::OpenApiRouter;

mod config;
mod processes;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    // Application state
    let ogcapi_config = ogcapi::services::Config {
        host: CONFIG.server.host.clone(),
        port: CONFIG.server.port,
        openapi: None,
        database_url: CONFIG.database.connection_string()?,
    };
    let ogcapi_state = ogcapi_services::AppState::new_from(&ogcapi_config).await;

    // Register processes/processors
    let ogcapi_state = ogcapi_state.processors(vec![
        Box::new(ogcapi_processes::echo::Echo),
        Box::new(processes::NDVIProcess),
        // Box::new(HelloProcess),
        // Box::new(Echo),
        // Box::new(GeoJsonLoader),
        // Box::new(GdalLoader),
    ]);

    // Build & run with hyper
    let mut ogcapi_service = ogcapi_services::Service::new_with(&ogcapi_config, ogcapi_state).await;

    // let router = OpenApiRouter::<AppState>::with_openapi(ApiDoc::openapi());
    // let router = OpenApiRouter::<AppState>::new();

    let router = OpenApiRouter::<AppState>::new();
    let (router, api) = router.split_for_parts();

    let router = router.route("/", get(|| async { "Hello, World!" }));

    ogcapi_service.router = ogcapi_service
        .router
        .nest("/test", router.with_state(AppState {}.clone()));

    ogcapi_service.serve().await;

    // middleware stack
    // let router = router.layer(
    //     ServiceBuilder::new()
    //         .set_x_request_id(MakeRequestUuid)
    //         .layer(SetSensitiveRequestHeadersLayer::new([
    //             AUTHORIZATION,
    //             PROXY_AUTHORIZATION,
    //             COOKIE,
    //             SET_COOKIE,
    //         ]))
    //         .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new()))
    //         .layer(CompressionLayer::new())
    //         .layer(CorsLayer::permissive())
    //         // .layer(CatchPanicLayer::custom(handle_panic))
    //         .propagate_x_request_id(),
    // );

    // let listener = TcpListener::bind((CONFIG.server.host.as_str(), CONFIG.server.port))
    //     .await
    //     .expect("create listener");

    // axum::serve::serve(listener, router.with_state(AppState {}.clone()))
    //     // .with_graceful_shutdown(shutdown_signal())
    //     .await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct AppState {}

// /// Custom panic handler
// fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response<Body> {
//     let details = if let Some(s) = err.downcast_ref::<String>() {
//         s.clone()
//     } else if let Some(s) = err.downcast_ref::<&str>() {
//         s.to_string()
//     } else {
//         "Unknown panic message".to_string()
//     };

//     // let body =
//     //     Exception::new_from_status(StatusCode::INTERNAL_SERVER_ERROR.as_u16()).detail(details);

//     // let body = serde_json::to_string(&body).unwrap();

//     Response::builder()
//         .status(StatusCode::INTERNAL_SERVER_ERROR)
//         .header(CONTENT_TYPE, "application/json")
//         // .body(Body::from(body))
//         .body(Body::from(details))
//         .unwrap()
// }
