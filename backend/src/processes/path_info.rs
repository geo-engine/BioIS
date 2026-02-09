#![allow(clippy::needless_for_each)] // TODO: remove when clippy is fixed for utoipa <https://github.com/juhaku/utoipa/issues/1420>
//! OpenAPI docs for processes.
//! The functions are placeholders only.

use crate::processes::ndvi::{NDVIProcessInputs, NDVIProcessOutputs};
use axum::Json;
use utoipa::OpenApi;

#[allow(unused, reason = "Placeholder for spec only")]
#[utoipa::path(
    post,
    path = "/processes/ndvi/execution",
    tag = "Processes",
    responses((status = OK, body = NDVIProcessOutputs))
)]
fn execute_ndvi(Json(_input): Json<NDVIProcessInputs>) {}

/// OpenAPI extension to include process endpoints in the generated documentation
#[allow(unused, reason = "Placeholder for spec only")]
#[derive(OpenApi)]
#[openapi(paths(execute_ndvi))]
pub struct ProcessesOpenApiSpec;
