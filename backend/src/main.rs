use crate::auth::GeoEngineAuthMiddlewareLayer;
use crate::collection_transactions::NoCollectionTransactions;
use crate::db::setup_db;
use crate::jobs::JobHandler;
use anyhow::Context;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Router, routing::get};
use config::CONFIG;
use geoengine_openapi_client::apis::configuration::Configuration;
use geoengine_openapi_client::apis::session_api::oidc_login;
use geoengine_openapi_client::models::{AuthCodeResponse, UserSession};
use ogcapi::services as ogcapi_services;
use ogcapi::services::Drivers;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod collection_transactions;
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

    let db_pool = setup_db(&CONFIG.database).await?;

    let drivers = Drivers {
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

    // Build & run with hyper
    let mut ogcapi_service =
        ogcapi_services::Service::try_new_with(&ogcapi_config, ogcapi_state).await?;

    let misc_router = Router::new()
        .route("/auth", get(auth_handler))
        .route("/health", get(health_handler))
        .with_state(CONFIG.geoengine.api_config(None));

    ogcapi_service.router = ogcapi_service.router.merge(misc_router);
    // ogcapi_service.router = ogcapi_service.router.layer(from_fn(auth_middleware));
    ogcapi_service.router = ogcapi_service.router.layer(GeoEngineAuthMiddlewareLayer);

    ogcapi_service.serve().await;

    Ok(())
}

async fn health_handler() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn auth_handler(
    State(api_config): State<Configuration>,
    Query(redirect_uri): Query<String>,
    Json(auth_code_response): Json<AuthCodeResponse>,
) -> ogcapi_services::Result<Json<UserSession>> {
    let user_session = oidc_login(&api_config, &redirect_uri, auth_code_response)
        .await
        .context("Failed to perform OIDC login")?;

    Ok(Json(user_session))
}

fn setup_tracing() {
    // setup tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(CONFIG.logging.clone().into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GeoEngineInstance;
    use axum::{body::Body, http::Request};
    use httptest::matchers::request::method;
    use httptest::{Expectation, Server, responders::json_encoded};
    use serde_json::json;
    use tower::ServiceExt;
    use url::Url;

    #[tokio::test]
    async fn test_health_route() {
        let app = Router::new().route("/health", get(health_handler));
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_auth_handler_with_mock_server() {
        // start mock server
        let server = Server::run();

        // respond to oidcLogin under an `/api` base with a valid user session
        server.expect(
            Expectation::matching(method("POST"))
                .respond_with(json_encoded(json!({
                    "id": "d1322969-5ada-4a2c-bacf-a3045383ba41",
                    "user": { "id": "9273bb02-95a6-49fe-b1c6-a32ff171d4a3", "email": "foo@example.com", "realName": "Max Muster" },
                    "created": "2020-01-01T00:00:00Z",
                    "validUntil": "2021-01-01T00:00:00Z",
                    "project": null,
                    "view": null,
                    "roles": []
                })))
        );

        let api_config = GeoEngineInstance {
            base_url: Url::parse(&server.url_str("")).expect("valid url"),
        }
        .api_config(None);

        // build test inputs
        let redirect = "http://example.com/redirect".to_string();
        let auth_code_response = AuthCodeResponse {
            code: String::new(),
            session_state: String::new(),
            state: String::new(),
        };

        // call handler
        let res = auth_handler(State(api_config), Query(redirect), Json(auth_code_response)).await;

        assert!(res.is_ok(), "expected Ok(UserSession) from auth_handler");
    }
}
