#![allow(clippy::needless_for_each)] // TODO: remove when clippy is fixed for utoipa <https://github.com/juhaku/utoipa/issues/1420>
//! OpenAPI docs for processes.
//! The functions are placeholders only.

use std::collections::HashMap;

use crate::processes::ndvi::{NDVIProcessInputs, NDVIProcessOutputs};
use axum::Json;
use ogcapi::types::processes::Response;
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

/// Process execution
#[allow(unused, reason = "Placeholder for spec only")]
// TODO: macro for generating this from the process definition
#[derive(Deserialize, ToSchema, Debug)]
pub struct NDVIProcessParams {
    pub inputs: NDVIProcessInputs,
    #[serde(default)]
    #[allow(clippy::zero_sized_map_values, reason = "Placeholder for spec only")]
    pub outputs: HashMap<String, ()>,
    #[serde(default)]
    pub response: Response,
}

#[allow(unused, reason = "Placeholder for spec only")]
#[utoipa::path(
    post,
    path = "/processes/ndvi/execution",
    tag = "Processes",
    responses((status = OK, body = NDVIProcessOutputs))
)]
fn execute_ndvi(Json(_input): Json<NDVIProcessParams>) {}

/// OpenAPI extension to include process endpoints in the generated documentation
#[allow(unused, reason = "Placeholder for spec only")]
#[derive(OpenApi)]
#[openapi(paths(execute_ndvi))]
pub struct ProcessesOpenApiSpec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processes_openapi_spec_is_valid() {
        let openapi = ProcessesOpenApiSpec::openapi();
        assert!(
            !openapi.paths.paths.is_empty(),
            "OpenAPI spec should contain paths"
        );
    }
}
