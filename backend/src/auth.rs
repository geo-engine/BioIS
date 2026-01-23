use crate::{config::CONFIG, util::Secret};
use anyhow::{Context, Result};
use axum::{
    extract::Request,
    http::{Response, StatusCode},
};
use futures::future::BoxFuture;
use geoengine_openapi_client::apis::{configuration, session_api::session_handler};
use nom::{
    IResult, Parser,
    bytes::{complete::tag_no_case, take},
    character::complete::space1,
    combinator::{all_consuming, map_res},
    sequence::separated_pair,
};
use tower_http::auth::AsyncAuthorizeRequest;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub session_token: Secret<Uuid>,
}

#[derive(Clone, Debug)]
pub struct GeoEngineAuthMiddleware {
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

impl GeoEngineAuthMiddleware {
    pub fn new() -> Self {
        let mut configuration = configuration::Configuration::new();
        configuration.base_path = CONFIG.geoengine.base_url.to_string();
        Self::from_configuration(configuration)
    }

    fn from_configuration(configuration: configuration::Configuration) -> Self {
        Self {
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
                prefix: vec!["/api", "/swagger"],
            },
        }
    }

    fn path_is_whitelisted(&self, path: &str) -> bool {
        self.whitelisted_paths.contains(path)
    }
}

impl AsyncAuthorizeRequest<axum::body::Body> for GeoEngineAuthMiddleware {
    type RequestBody = axum::body::Body;
    type ResponseBody = axum::body::Body;
    type Future =
        BoxFuture<'static, Result<Request<Self::RequestBody>, Response<Self::ResponseBody>>>;

    fn authorize(&mut self, mut request: Request<Self::RequestBody>) -> Self::Future {
        if self.path_is_whitelisted(request.uri().path()) {
            return Box::pin(async move { Ok(request) });
        }

        let mut configuration = self.configuration.clone();
        Box::pin(async move {
            let Some(auth_header) = request
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| parse_bearer_token(h).ok())
            else {
                return Err(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Default::default())
                    .expect("to build empty response"));
            };

            configuration.bearer_access_token = Some(auth_header.to_string());

            let Ok(session) = session_handler(&configuration).await else {
                return Err(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Default::default())
                    .expect("to build empty response"));
            };

            let user = User {
                id: session.user.id,
                session_token: session.id.into(),
            };
            request.extensions_mut().insert(user);

            Ok(request)
        })
    }
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
    use httptest::matchers::*;
    use httptest::responders::*;
    use httptest::{Expectation, Server};
    use serde_json::json;

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
        let middleware = GeoEngineAuthMiddleware::new();

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
        let mut middleware = GeoEngineAuthMiddleware::new();

        let http_req: HttpRequest<Body> = HttpRequest::builder()
            .uri("/private")
            .body(Body::empty())
            .expect("to build http request");

        let result = middleware.authorize(http_req).await;

        assert!(
            result.is_err(),
            "expected an Err(Response) when header missing"
        );
        let resp = result.unwrap_err();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn authorize_with_valid_header_inserts_user_extension() {
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
        let mut middleware = GeoEngineAuthMiddleware::from_configuration(configuration);

        let token = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        let header_value = format!("Bearer {token}");

        let http_req: HttpRequest<Body> = HttpRequest::builder()
            .uri("/private")
            .header("Authorization", header_value)
            .body(Body::empty())
            .expect("to build http request");

        let result = middleware.authorize(http_req).await;
        assert!(result.is_ok(), "expected Ok(Request) for valid session");

        let req = result.unwrap();
        let user = req.extensions().get::<User>().expect("user in extensions");

        let expected_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let expected_session_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

        assert_eq!(user.id, expected_user_id);
        assert_eq!(*user.session_token, expected_session_id);

        // `httptest::Server` stops when dropped at test end
    }
}
