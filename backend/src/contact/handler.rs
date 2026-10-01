use super::mailer::{Mailer, smtp_transport};
use crate::{
    config::{ContactConfig, ContactRateLimitConfig},
    server::AppState,
};
use anyhow::Context;
use axum::{
    BoxError, Json,
    error_handling::HandleErrorLayer,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use indoc::formatdoc;
use lettre::{
    Address, Message,
    message::{Mailbox, header::ContentType},
};
use ogcapi::{services as ogcapi_services, types::common::Exception};
use serde::Deserialize;
use std::{fmt, sync::Arc};
use tower::{
    ServiceBuilder, buffer::BufferLayer, limit::RateLimitLayer, load_shed::LoadShedLayer,
    load_shed::error::Overloaded,
};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

const MAX_SHORT_FIELD_LENGTH: usize = 200;
const MAX_MESSAGE_LENGTH: usize = 5000;
/// Number of requests that can be queued for the rate limiter.
const RATE_LIMIT_BUFFER_SIZE: usize = 64;

#[derive(Clone)]
struct ContactState {
    mailer: Arc<dyn Mailer>,
    from: Mailbox,
    to: Mailbox,
}

impl ContactState {
    fn from_config(config: &ContactConfig, mailer: Arc<dyn Mailer>) -> anyhow::Result<Self> {
        Ok(Self {
            mailer,
            from: config
                .from
                .parse()
                .context("invalid `contact.from` address")?,
            to: config.to.parse().context("invalid `contact.to` address")?,
        })
    }
}

pub fn router(config: &ContactConfig) -> anyhow::Result<OpenApiRouter<AppState>> {
    let mailer = Arc::new(smtp_transport(&config.smtp)?);
    router_with_mailer(config, mailer)
}

fn router_with_mailer<S>(
    config: &ContactConfig,
    mailer: Arc<dyn Mailer>,
) -> anyhow::Result<OpenApiRouter<S>>
where
    S: Clone + Send + Sync + 'static,
{
    let ContactRateLimitConfig { max_requests, .. } = config.rate_limit;

    Ok(OpenApiRouter::new()
        .routes(routes!(submit_contact))
        .route_layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_rate_limit_error))
                // makes the service `Clone`
                .layer(BufferLayer::new(RATE_LIMIT_BUFFER_SIZE))
                // rejects requests instead of waiting for the rate limit to reset
                .layer(LoadShedLayer::new())
                .layer(RateLimitLayer::new(
                    max_requests,
                    config.rate_limit.window(),
                )),
        )
        .with_state(ContactState::from_config(config, mailer)?))
}

async fn handle_rate_limit_error(error: BoxError) -> Response {
    let exception = if error.is::<Overloaded>() {
        Exception::new_from_status(StatusCode::TOO_MANY_REQUESTS.as_u16())
            .detail("Too many requests. Please try again later.")
    } else {
        tracing::error!("Rate limiting failed: {error}");
        Exception::new_from_status(StatusCode::INTERNAL_SERVER_ERROR.as_u16())
    };
    ogcapi_services::Error::from(exception).into_response()
}

/// A request for pilot access, submitted via the contact form on the landing page.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContactRequest {
    pub name: String,
    #[schema(format = "email")]
    pub email: String,
    pub organization: String,
    pub role: ContactRole,
    /// Optional message, may be empty.
    pub message: String,
    /// Must be `true`: the user agrees to the processing of their data.
    pub privacy_consent: bool,
    /// Must be left empty. Used to detect spam bots.
    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ContactRole {
    /// Large corporate (CSRD / ESRS E4)
    LargeCorporate,
    /// SME (VSME B5)
    Sme,
    /// ESG consultancy
    Consultancy,
    Other,
}

impl fmt::Display for ContactRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::LargeCorporate => "Large corporate (CSRD / ESRS E4)",
            Self::Sme => "SME (VSME B5)",
            Self::Consultancy => "ESG consultancy",
            Self::Other => "Other",
        })
    }
}

/// Sends a request for pilot access to the `BioIS` team.
#[utoipa::path(post, path = "/", tag = "Contact",
    request_body = ContactRequest,
    responses(
        (
            status = NO_CONTENT,
            description = "The request was sent to the BioIS team.",
        ),
        (
            status = BAD_REQUEST,
            description = "The request is invalid.",
            body = Exception,
            example = json!(Exception::new_from_status(400))
        ),
        (
            status = TOO_MANY_REQUESTS,
            description = "Too many requests. Please try again later.",
            body = Exception,
            example = json!(Exception::new_from_status(429))
        ),
        (
            status = INTERNAL_SERVER_ERROR,
            description = "A server error occurred.",
            body = Exception,
            example = json!(Exception::new_from_status(500))
        )
    )
)]
async fn submit_contact(
    State(state): State<ContactState>,
    Json(request): Json<ContactRequest>,
) -> ogcapi_services::Result<StatusCode> {
    if request.website.as_deref().is_some_and(|w| !w.is_empty()) {
        tracing::debug!("Ignoring contact request with filled honeypot field");
        return Ok(StatusCode::NO_CONTENT);
    }

    let request = ValidContactRequest::try_from(request)
        .map_err(|detail| Exception::new_from_status(400).detail(detail))?;
    let message = request.to_message(&state.from, &state.to)?;

    state.mailer.send(message).await?;
    tracing::info!("Sent contact request to {}", state.to.email);

    Ok(StatusCode::NO_CONTENT)
}

/// A [`ContactRequest`] with trimmed and validated fields.
#[derive(Debug)]
struct ValidContactRequest {
    name: String,
    email: Address,
    organization: String,
    role: ContactRole,
    message: String,
}

impl TryFrom<ContactRequest> for ValidContactRequest {
    /// A message describing why the request is invalid.
    type Error = String;

    fn try_from(request: ContactRequest) -> Result<Self, Self::Error> {
        if !request.privacy_consent {
            return Err("Consent to the privacy policy is required.".to_string());
        }

        let name = validate_text("name", &request.name, MAX_SHORT_FIELD_LENGTH)?;
        let organization = validate_text(
            "organization",
            &request.organization,
            MAX_SHORT_FIELD_LENGTH,
        )?;
        let message = validate_optional_text("message", &request.message, MAX_MESSAGE_LENGTH)?;
        let email = request
            .email
            .trim()
            .parse::<Address>()
            .map_err(|_| "The email address is invalid.".to_string())?;

        Ok(Self {
            name,
            email,
            organization,
            role: request.role,
            message,
        })
    }
}

fn validate_text(field: &str, value: &str, max_length: usize) -> Result<String, String> {
    let value = validate_optional_text(field, value, max_length)?;
    if value.is_empty() {
        return Err(format!("The {field} is required."));
    }
    Ok(value)
}

fn validate_optional_text(field: &str, value: &str, max_length: usize) -> Result<String, String> {
    let value = value.trim();
    if value.chars().count() > max_length {
        return Err(format!(
            "The {field} must not exceed {max_length} characters."
        ));
    }
    Ok(value.to_string())
}

impl ValidContactRequest {
    fn to_message(&self, from: &Mailbox, to: &Mailbox) -> anyhow::Result<Message> {
        let Self {
            name,
            email,
            organization,
            role,
            message,
        } = self;
        let message = if message.is_empty() {
            "(no message)"
        } else {
            message
        };

        let body = formatdoc! {"
            New request for BioIS pilot access:

            Name:         {name}
            Email:        {email}
            Organization: {organization}
            Role:         {role}

            Message:
            {message}
        "};

        Message::builder()
            .from(from.clone())
            .to(to.clone())
            .reply_to(Mailbox::new(Some(name.clone()), email.clone()))
            .subject(format!("BioIS pilot access request – {organization}"))
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .context("failed to build contact email")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SmtpConfig, SmtpTls};
    use axum::{body::Body, http::Request};
    use lettre::transport::stub::AsyncStubTransport;
    use serde_json::json;
    use tower::ServiceExt;

    fn config(max_requests: u64) -> ContactConfig {
        ContactConfig {
            from: "BioIS <noreply@example.com>".to_string(),
            to: "info@example.com".to_string(),
            smtp: SmtpConfig {
                host: "localhost".to_string(),
                port: 25,
                tls: SmtpTls::None,
                helo_name: None,
                username: None,
                password: None,
            },
            rate_limit: ContactRateLimitConfig {
                max_requests,
                window_seconds: 3600,
            },
        }
    }

    fn valid_request() -> serde_json::Value {
        json!({
            "name": "Max Muster",
            "email": "max@example.com",
            "organization": "ACME Corp",
            "role": "largeCorporate",
            "message": "We would like to join the pilot.",
            "privacyConsent": true,
            "website": ""
        })
    }

    fn app(mailer: Arc<AsyncStubTransport>, max_requests: u64) -> axum::Router {
        let (router, api) = OpenApiRouter::new()
            .nest(
                "/contact",
                router_with_mailer(&config(max_requests), mailer).unwrap(),
            )
            .split_for_parts();
        assert!(
            api.paths.paths.contains_key("/contact"),
            "{:?}",
            api.paths.paths.keys()
        );
        router
    }

    async fn post(app: &axum::Router, body: &serde_json::Value) -> StatusCode {
        let request = Request::builder()
            .method("POST")
            .uri("/contact")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn it_sends_email_for_valid_request() {
        let mailer = Arc::new(AsyncStubTransport::new_ok());
        let app = app(mailer.clone(), 5);

        let status = post(&app, &valid_request()).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        let messages = mailer.messages().await;
        assert_eq!(messages.len(), 1);
        let (envelope, email) = &messages[0];
        assert_eq!(envelope.to(), ["info@example.com".parse().unwrap()]);
        assert!(email.contains("Reply-To: \"Max Muster\" <max@example.com>"));
        assert!(email.contains("Subject: BioIS pilot access request"));
        assert!(email.contains("We would like to join the pilot."));
    }

    #[tokio::test]
    async fn it_sends_email_without_message() {
        let mailer = Arc::new(AsyncStubTransport::new_ok());
        let app = app(mailer.clone(), 5);

        let mut request = valid_request();
        request["message"] = json!("  ");

        let status = post(&app, &request).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        let messages = mailer.messages().await;
        assert_eq!(messages.len(), 1);
        assert!(messages[0].1.contains("(no message)"));
    }

    #[tokio::test]
    async fn it_silently_drops_honeypot_requests() {
        let mailer = Arc::new(AsyncStubTransport::new_ok());
        let app = app(mailer.clone(), 5);

        let mut request = valid_request();
        request["website"] = json!("https://spam.example.com");

        let status = post(&app, &request).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(mailer.messages().await.is_empty());
    }

    #[tokio::test]
    async fn it_rejects_invalid_requests() {
        let mailer = Arc::new(AsyncStubTransport::new_ok());
        let app = app(mailer.clone(), 10);

        let mut no_consent = valid_request();
        no_consent["privacyConsent"] = json!(false);
        let mut bad_email = valid_request();
        bad_email["email"] = json!("not-an-email");
        let mut empty_name = valid_request();
        empty_name["name"] = json!("   ");
        let mut long_message = valid_request();
        long_message["message"] = json!("a".repeat(MAX_MESSAGE_LENGTH + 1));

        for request in [no_consent, bad_email, empty_name, long_message] {
            let status = post(&app, &request).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "request: {request}");
        }
        assert!(mailer.messages().await.is_empty());
    }

    #[tokio::test]
    async fn it_fails_if_email_cannot_be_sent() {
        let app = app(Arc::new(AsyncStubTransport::new_error()), 5);

        let status = post(&app, &valid_request()).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn it_rate_limits_requests() {
        let mailer = Arc::new(AsyncStubTransport::new_ok());
        let app = app(mailer.clone(), 2);

        for _ in 0..2 {
            let status = post(&app, &valid_request()).await;
            assert_eq!(status, StatusCode::NO_CONTENT);
        }

        let status = post(&app, &valid_request()).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);

        assert_eq!(mailer.messages().await.len(), 2);
    }
}
