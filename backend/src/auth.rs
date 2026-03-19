use std::convert::Infallible;

use crate::{
    config::CONFIG,
    state::USER,
    util::{Secret, error_response},
};
use anyhow::{Context, Result};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures::future::BoxFuture;
use geoengine_api_client::apis::{configuration, session_api::session_handler};
use nom::{
    IResult, Parser,
    bytes::{complete::tag_no_case, take},
    character::complete::space1,
    combinator::{all_consuming, map_res},
    sequence::separated_pair,
};
use ogcapi::types::common::Exception;
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub session_token: Secret<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthCodeResponse {
    pub code: String,
    pub session_state: String,
    pub state: String,
}

impl From<AuthCodeResponse> for geoengine_api_client::models::AuthCodeResponse {
    fn from(value: AuthCodeResponse) -> Self {
        Self {
            code: value.code,
            session_state: value.session_state,
            state: value.state,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserSession {
    pub created: String,
    pub id: Uuid,
    pub roles: Vec<Uuid>,
    pub user: UserInfo,
    pub valid_until: String,
}

impl From<geoengine_api_client::models::UserSession> for UserSession {
    fn from(value: geoengine_api_client::models::UserSession) -> Self {
        Self {
            created: value.created,
            id: value.id,
            roles: value.roles,
            user: UserInfo {
                email: value.user.email.flatten(),
                id: value.user.id,
                real_name: value.user.real_name.flatten(),
            },
            valid_until: value.valid_until,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub email: Option<String>,
    pub id: uuid::Uuid,
    pub real_name: Option<String>,
}

#[derive(Clone)]
pub struct GeoEngineAuthMiddlewareLayer;

impl<S> Layer<S> for GeoEngineAuthMiddlewareLayer {
    type Service = GeoEngineAuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GeoEngineAuthMiddleware::new(inner)
    }
}

#[derive(Clone, Debug)]
pub struct GeoEngineAuthMiddleware<S> {
    inner: S,
    configuration: configuration::Configuration,
    whitelisted_paths: WhitelistedPaths,
}

#[derive(Clone, Debug)]
struct WhitelistedPaths {
    exact: Vec<&'static str>,
    prefix: Vec<&'static str>,
}

impl WhitelistedPaths {
    fn contains(&self, path: &str) -> bool {
        self.exact.contains(&path) || self.prefix.iter().any(|&p| path.starts_with(p))
    }
}

impl<S> GeoEngineAuthMiddleware<S> {
    pub fn new(inner: S) -> Self {
        Self::from_configuration(inner, CONFIG.geoengine.api_config(None))
    }

    pub fn from_configuration(inner: S, configuration: configuration::Configuration) -> Self {
        Self {
            inner,
            configuration,
            whitelisted_paths: WhitelistedPaths {
                exact: vec![
                    "/",
                    "/conformance",
                    "/health",
                    "/processes",
                    "/processes/echo",
                    "/processes/ndvi",
                ],
                prefix: vec!["/api", "/swagger", "/auth/"],
            },
        }
    }

    fn path_is_whitelisted(&self, path: &str) -> bool {
        self.whitelisted_paths.contains(path)
    }
}

impl<S> Service<Request> for GeoEngineAuthMiddleware<S>
where
    S: Service<
            Request<axum::body::Body>,
            Response = axum::http::Response<axum::body::Body>,
            Error = Infallible,
        > + Send
        + Clone
        + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        if self.path_is_whitelisted(request.uri().path()) {
            return Box::pin(self.inner.call(request));
        }

        let mut inner = self.inner.clone();
        let mut configuration = self.configuration.clone();
        Box::pin(async move {
            let auth_header = match bearer_token_from_header(request.headers()) {
                Ok(auth_header) => auth_header,
                Err(error) => {
                    let status = StatusCode::UNAUTHORIZED;
                    let exception = Exception::new_from_status(status.as_u16()).detail(error);
                    return Ok((status, exception.to_string()).into_response());
                }
            };

            configuration.bearer_access_token = Some(auth_header.into());

            let session = match session_handler(&configuration).await {
                Ok(session) => session,
                Err(error) => {
                    let (error_msg, status_code) = match &error {
                        geoengine_api_client::apis::Error::ResponseError(e) => (
                            error_response(&error)
                                .map(|e| e.message)
                                .unwrap_or_default(),
                            e.status,
                        ),
                        error => (error.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
                    };

                    let exception =
                        Exception::new_from_status(status_code.as_u16()).detail(error_msg);
                    return Ok((status_code, exception.to_string()).into_response());
                }
            };

            let user = User {
                id: session.user.id,
                session_token: session.id.into(),
            };

            // continue the request, scoped
            USER.scope(user, inner.call(request)).await
        })
    }
}

fn bearer_token_from_header(headers: &HeaderMap) -> Result<Uuid> {
    let auth_header = headers
        .get("Authorization")
        .context("Missing Authorization header")?
        .to_str()
        .context("Invalid Authorization header")?;

    parse_bearer_token(auth_header)
}

fn parse_bearer_token(header_value: &str) -> Result<Uuid> {
    bearer_token_parser(header_value)
        .map(|(_, token)| token)
        .map_err(|e| e.map_input(ToString::to_string))
        .context("Failed to parse bearer token")
}

fn bearer_token_parser(header_value: &str) -> IResult<&str, Uuid> {
    let (_, (_, token)) = all_consuming(separated_pair(tag_no_case("Bearer"), space1, uuid_parser))
        .parse(header_value)?;
    Ok((header_value, token))
}

fn uuid_parser(input: &str) -> IResult<&str, Uuid> {
    // A standard hyphenated UUID is exactly 36 characters
    map_res(take(36usize), Uuid::parse_str).parse(input)
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use axum::response::Response;
    use geoengine_api_client::apis::configuration;
    use httptest::matchers::*;
    use httptest::responders::*;
    use httptest::{Expectation, Server};
    use serde_json::json;
    use tower::util::BoxCloneService;

    #[test]
    fn it_parses_bearer_tokens() {
        let token = Uuid::new_v4();
        let header_value = format!("Bearer {token}");
        let parsed_token = parse_bearer_token(&header_value).expect("to parse token");
        assert_eq!(parsed_token, token);
    }

    #[test]
    fn it_parses_bearer_case_insensitive() {
        let token = Uuid::new_v4();
        let header_value = format!("bearer {token}");
        let parsed_token = parse_bearer_token(&header_value).expect("to parse token");
        assert_eq!(parsed_token, token);
    }

    #[test]
    fn parse_fails_on_invalid_token() {
        let header_value = "Bearer not-a-uuid";
        let err = parse_bearer_token(header_value).unwrap_err();
        assert!(err.to_string().contains("Failed to parse bearer token"));
    }

    #[test]
    fn uuid_parser_rejects_short_input() {
        let input = "123";
        let res = uuid_parser(input);
        assert!(res.is_err());
    }

    #[test]
    fn whitelisted_paths_match_exact_and_prefix() {
        let middleware = GeoEngineAuthMiddleware::new(Request::<()>::default());

        // exact
        assert!(middleware.path_is_whitelisted("/"));
        assert!(middleware.path_is_whitelisted("/health"));
        assert!(middleware.path_is_whitelisted("/processes/echo"));

        // prefix
        assert!(middleware.path_is_whitelisted("/api/some/resource"));
        assert!(middleware.path_is_whitelisted("/swagger/index.html"));

        // not whitelisted
        assert!(!middleware.path_is_whitelisted("/private"));
    }

    #[tokio::test]
    async fn authorize_returns_unauthorized_for_missing_header() {
        let mut middleware = GeoEngineAuthMiddleware::new(mock_inner());

        let http_req: HttpRequest<Body> = HttpRequest::builder()
            .uri("/private")
            .body(Body::empty())
            .expect("to build http request");

        let result = middleware.call(http_req).await;

        let respons = result.unwrap();
        assert_eq!(respons.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            exception_from_body(respons.into_body())
                .await
                .detail
                .unwrap(),
            "Missing Authorization header"
        );
    }

    async fn exception_from_body(body: Body) -> Exception {
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .expect("to read response body");
        serde_json::from_slice(&body_bytes).expect("to deserialize Exception")
    }

    fn mock_inner() -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
        let inner =
            tower::service_fn(|_req: Request<Body>| async { Ok(StatusCode::OK.into_response()) });
        tower::util::BoxCloneService::new(inner)
    }

    fn mock_inner_outputs_user() -> BoxCloneService<Request<Body>, Response<Body>, Infallible> {
        let inner = tower::service_fn(|_req: Request<Body>| async {
            Ok((StatusCode::OK, USER.get().id.to_string()).into_response())
        });
        tower::util::BoxCloneService::new(inner)
    }

    #[tokio::test]
    async fn authorize_with_valid_header_creates_user_scope() {
        // start mock auth service using `httptest`
        let server = Server::run();

        // Respond with a valid session for any GET (works regardless of path formatting)
        server.expect(
            Expectation::matching(request::method("GET"))
                .respond_with(json_encoded(json!({
                    "id": "00000000-0000-0000-0000-000000000002",
                    "user": { "id": "00000000-0000-0000-0000-000000000001", "email": null, "realName": null },
                    "created": "2021-01-01T00:00:00Z",
                    "validUntil": "2022-01-01T00:00:00Z",
                    "project": null,
                    "view": null,
                    "roles": []
                })))
        );

        let mut configuration = configuration::Configuration::new();
        configuration.base_path = server.url_str("");
        let mut middleware =
            GeoEngineAuthMiddleware::from_configuration(mock_inner_outputs_user(), configuration);

        let token = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let header_value = format!("Bearer {token}");

        let http_req: HttpRequest<Body> = HttpRequest::builder()
            .uri("/private")
            .header("Authorization", header_value)
            .body(Body::empty())
            .expect("to build http request");

        let result = middleware.call(http_req).await;
        assert!(result.is_ok(), "expected Ok(Request) for valid session");

        let response = result.unwrap();
        assert!(
            response.status().is_success(),
            "expected successful response"
        );
        let user_id = body_to_string(response.into_body()).await;

        let expected_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

        assert_eq!(user_id, expected_user_id.to_string());

        // `httptest::Server` stops when dropped at test end
    }

    async fn body_to_string(body: Body) -> String {
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .expect("to read response body");
        String::from_utf8(body_bytes.to_vec()).expect("to convert body to string")
    }

    #[tokio::test]
    async fn it_returns_401_on_invalid_token() {
        const INVALID_BEARER_TOKEN: &str = "00000000-0000-0000-0000-000000000001";

        // start mock auth service using `httptest`
        let server = Server::run();

        // Respond with a valid session for any GET (works regardless of path formatting)
        server.expect(
            Expectation::matching(request::method("GET")).respond_with(
                status_code(401)
                    .append_header("Content-Type", "application/json")
                    .body(
                        json!({
                            "error": "Unauthorized",
                            "message": "Authorization error: The session id is invalid."
                        })
                        .to_string(),
                    ),
            ),
        );

        let mut configuration = configuration::Configuration::new();
        configuration.base_path = server.url_str("");
        let mut middleware =
            GeoEngineAuthMiddleware::from_configuration(mock_inner_outputs_user(), configuration);

        let http_req: HttpRequest<Body> = HttpRequest::builder()
            .uri("/private")
            .header("Authorization", format!("Bearer {INVALID_BEARER_TOKEN}"))
            .body(Body::empty())
            .expect("to build http request");

        let result = middleware.call(http_req).await;

        let response = result.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let msg = exception_from_body(response.into_body())
            .await
            .detail
            .unwrap();
        assert_eq!(msg, "Authorization error: The session id is invalid.");
    }
}
