#![allow(clippy::needless_for_each)] // TODO: remove when clippy is fixed for utoipa <https://github.com/juhaku/utoipa/issues/1420>
//! OpenAPI docs for processes.
//! The functions are placeholders only.

use std::collections::HashMap;

use crate::processes::{
    biodiversity_sensitive_areas::{
        BiodiversitySensitiveAreasProcessInputs, BiodiversitySensitiveAreasProcessOutputs,
    },
    climate_risk::{ClimateRiskInputs, ClimateRiskRawRow, ClimateRiskRow},
    habitat_distance::{HabitatDistanceProcessInputs, HabitatDistanceProcessOutputs},
    land_use_sealed_area::{LandUseSealedAreaProcessInputs, LandUseSealedAreaProcessOutputs},
    ndvi::{NDVIProcessInputs, NDVIProcessOutputs},
    parameters::{DataResource, DataResourceSchema},
};
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

/// Process execution
#[allow(unused, reason = "Placeholder for spec only")]
// TODO: macro for generating this from the process definition
#[derive(Deserialize, ToSchema, Debug)]
pub struct HabitatDistanceProcessParams {
    pub inputs: HabitatDistanceProcessInputs,
    #[serde(default)]
    #[allow(clippy::zero_sized_map_values, reason = "Placeholder for spec only")]
    pub outputs: HashMap<String, ()>,
    #[serde(default)]
    pub response: Response,
}

#[allow(unused, reason = "Placeholder for spec only")]
#[utoipa::path(
    post,
    path = "/processes/habitatDistance/execution",
    tag = "Processes",
    responses((status = OK, body = HabitatDistanceProcessOutputs))
)]
fn execute_habitat_distance(Json(_input): Json<HabitatDistanceProcessParams>) {}

/// Process execution (Biodiversity Sensitive Areas – ESRS E4-5)
#[allow(unused, reason = "Placeholder for spec only")]
#[derive(Deserialize, ToSchema, Debug)]
pub struct BiodiversitySensitiveAreasProcessParams {
    pub inputs: BiodiversitySensitiveAreasProcessInputs,
    #[serde(default)]
    #[allow(clippy::zero_sized_map_values, reason = "Placeholder for spec only")]
    pub outputs: HashMap<String, ()>,
    #[serde(default)]
    pub response: Response,
}

#[allow(unused, reason = "Placeholder for spec only")]
#[utoipa::path(
    post,
    path = "/processes/biodiversity-sensitive-areas/execution",
    tag = "Processes",
    responses((status = OK, body = BiodiversitySensitiveAreasProcessOutputs))
)]
fn execute_biodiversity_sensitive_areas(
    Json(_input): Json<BiodiversitySensitiveAreasProcessParams>,
) {
}

/// Process execution (Climate Risk)
#[allow(unused, reason = "Placeholder for spec only")]
#[derive(Deserialize, ToSchema, Debug)]
pub struct ClimateRiskProcessParams {
    pub inputs: ClimateRiskInputs,
    #[serde(default)]
    #[allow(clippy::zero_sized_map_values, reason = "Placeholder for spec only")]
    pub outputs: HashMap<String, ()>,
    #[serde(default)]
    pub response: Response,
}

/// Response body of a climate-risk execution: one summary table per computed scenario, plus the
/// optional `inputs` echo and `rawEnsembleData`. Which scenario keys are present depends on the
/// requested outputs and on the model registry (`conf/nexgddp_models.toml`); the process
/// description's `outputs` is the authoritative list for a given deployment.
#[allow(unused, reason = "Placeholder for spec only")]
#[derive(ToSchema, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClimateRiskProcessResponses {
    /// Summary table for SSP2-4.5.
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub ssp245: Option<DataResource<Vec<ClimateRiskRow>>>,
    /// Summary table for SSP3-7.0.
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub ssp370: Option<DataResource<Vec<ClimateRiskRow>>>,
    /// Summary table for SSP5-8.5.
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub ssp585: Option<DataResource<Vec<ClimateRiskRow>>>,
    /// Per-model raw values for each variable x scenario combination.
    #[schema(value_type = Option<DataResourceSchema>, inline)]
    pub raw_ensemble_data: Option<DataResource<Vec<ClimateRiskRawRow>>>,
    /// The submitted inputs, echoed back.
    pub inputs: Option<ClimateRiskInputs>,
}

#[allow(unused, reason = "Placeholder for spec only")]
#[utoipa::path(
    post,
    path = "/processes/climate-risk/execution",
    tag = "Processes",
    responses((status = OK, body = ClimateRiskProcessResponses))
)]
fn execute_climate_risk(Json(_input): Json<ClimateRiskProcessParams>) {}

/// Process execution (Land Use Sealed Area – ESRS E4-5)
#[allow(unused, reason = "Placeholder for spec only")]
#[derive(Deserialize, ToSchema, Debug)]
pub struct LandUseSealedAreaProcessParams {
    pub inputs: LandUseSealedAreaProcessInputs,
    #[serde(default)]
    #[allow(clippy::zero_sized_map_values, reason = "Placeholder for spec only")]
    pub outputs: HashMap<String, ()>,
    #[serde(default)]
    pub response: Response,
}

#[allow(unused, reason = "Placeholder for spec only")]
#[utoipa::path(
    post,
    path = "/processes/land-use-sealed-area/execution",
    tag = "Processes",
    responses((status = OK, body = LandUseSealedAreaProcessOutputs))
)]
fn execute_land_use_sealed_area(Json(_input): Json<LandUseSealedAreaProcessParams>) {}

/// OpenAPI extension to include process endpoints in the generated documentation
#[allow(unused, reason = "Placeholder for spec only")]
#[derive(OpenApi)]
#[openapi(paths(
    execute_ndvi,
    execute_habitat_distance,
    execute_biodiversity_sensitive_areas,
    execute_climate_risk,
    execute_land_use_sealed_area
))]
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
