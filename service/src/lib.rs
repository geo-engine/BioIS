use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(health, get_indicators, calculate_indicator),
    components(schemas(IndicatorRequest, IndicatorResponse, HealthResponse, Indicator))
)]
pub struct ApiDoc;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Indicator {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct IndicatorRequest {
    /// Geographic bounding box coordinates
    #[schema(example = "-180,-90,180,90")]
    pub bbox: String,
    /// Indicator type to calculate
    #[schema(example = "species_richness")]
    pub indicator_type: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct IndicatorResponse {
    pub indicator_type: String,
    pub value: f64,
    pub bbox: String,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get available biodiversity indicators
#[utoipa::path(
    get,
    path = "/indicators",
    responses(
        (status = 200, description = "List of available indicators", body = Vec<Indicator>)
    )
)]
pub async fn get_indicators() -> Json<Vec<Indicator>> {
    Json(vec![
        Indicator {
            name: "species_richness".to_string(),
            description: "Number of different species in an area".to_string(),
        },
        Indicator {
            name: "biodiversity_index".to_string(),
            description: "Shannon diversity index for the area".to_string(),
        },
    ])
}

/// Calculate a biodiversity indicator for a given area
#[utoipa::path(
    post,
    path = "/indicators/calculate",
    request_body = IndicatorRequest,
    responses(
        (status = 200, description = "Calculated indicator value", body = IndicatorResponse)
    )
)]
pub async fn calculate_indicator(
    Json(payload): Json<IndicatorRequest>,
) -> Result<Json<IndicatorResponse>, StatusCode> {
    // Mock calculation - in real implementation, this would query geoengine API
    let value = match payload.indicator_type.as_str() {
        "species_richness" => 42.0,
        "biodiversity_index" => 2.3,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    Ok(Json(IndicatorResponse {
        indicator_type: payload.indicator_type,
        value,
        bbox: payload.bbox,
    }))
}

pub fn create_app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/indicators", get(get_indicators))
        .route("/indicators/calculate", post(calculate_indicator))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
}
