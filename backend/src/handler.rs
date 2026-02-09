use crate::auth::{AuthCodeResponse, UserSession};
use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    routing::MethodRouter,
};
use geoengine_openapi_client::apis::{configuration::Configuration, session_api::oidc_login};
use ogcapi::{services as ogcapi_services, types::common::Exception};
use utoipa::openapi::{Paths, RefOr, Schema};
use utoipa_axum::routes;

type Routes = (
    Vec<(String, RefOr<Schema>)>,
    Paths,
    MethodRouter<Configuration>,
);

pub fn routes() -> Routes {
    routes!(health_handler, auth_handler)
}

#[utoipa::path(get, path = "/health", responses((status = NO_CONTENT)))]
pub async fn health_handler() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[utoipa::path(post, path = "/auth", tag = "User",
    responses(
        (
            status = OK,
            description = "The OIDC login flow was successful, and a user session has been created.",
            body = UserSession
        ),
        (
            status = INTERNAL_SERVER_ERROR,
            description = "A server error occurred.", 
            body = Exception,
            example = json!(Exception::new_from_status(500))
        )
    )
)]
pub async fn auth_handler(
    State(api_config): State<Configuration>,
    Query(redirect_uri): Query<String>,
    Json(auth_code_response): Json<AuthCodeResponse>,
) -> ogcapi_services::Result<Json<UserSession>> {
    let user_session = oidc_login(&api_config, &redirect_uri, auth_code_response.into())
        .await
        .context("Failed to perform OIDC login")?;

    Ok(Json(user_session.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthCodeResponse;
    use crate::config::GeoEngineInstance;
    use axum::extract::{Query, State};
    use axum::routing::get;
    use axum::{Json, Router};
    use axum::{body::Body, http::Request};
    use httptest::matchers::request::method;
    use httptest::{Expectation, Server, responders::json_encoded};
    use reqwest::StatusCode;
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
