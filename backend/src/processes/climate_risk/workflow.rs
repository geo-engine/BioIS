use geoengine_api_client::models::{GdalSourceParameters, MultiBandGdalSource, RasterOperator};

use super::{ClimateRiskProcess, types::*};
impl ClimateRiskProcess {
    /// The precomputed yearly event-day counts of one indicator for a model/scenario: a
    /// single-band time series with one slice per year, built by
    /// `notebooks/nexgddp_cmip6_aggregate_yearly.ipynb`. `GeoEngine` stores datasets created from a
    /// workflow as `MultiBandGdalSource` datasets.
    pub(crate) fn yearly_count_source(
        variable: ClimateVariable,
        model: &ClimateModelProperties,
        scenario: &ClimateScenarioProperties,
    ) -> RasterOperator {
        RasterOperator::MultiBandGdalSource(
            MultiBandGdalSource {
                r#type: Default::default(),
                params: GdalSourceParameters {
                    data: yearly_dataset_name(variable, model, scenario),
                    overview_level: None,
                }
                .into(),
            }
            .into(),
        )
    }
}

/// `nexgddp_cmip6_{model}_{scenario}_{indicator}_yearly`, the name the aggregation notebook gives
/// each yearly dataset.
pub(crate) fn yearly_dataset_name(
    variable: ClimateVariable,
    model: &ClimateModelProperties,
    scenario: &ClimateScenarioProperties,
) -> String {
    format!(
        "nexgddp_cmip6_{}_{}_{}_yearly",
        model.id,
        scenario.dataset_prefix,
        variable.name()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_the_yearly_dataset_of_an_indicator() {
        let model = ClimateModelProperties {
            id: "ACCESS-CM2".to_string(),
            variant: "r1i1p1f1".to_string(),
            grid: "gn".to_string(),
            scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
        };

        let source = ClimateRiskProcess::yearly_count_source(
            ClimateVariable::HeatDays,
            &model,
            &ClimateScenario::Ssp245.properties(),
        );

        assert!(matches!(source, RasterOperator::MultiBandGdalSource(_)));
        let value = serde_json::to_value(&source).unwrap();
        assert_eq!(
            value["params"]["data"],
            "nexgddp_cmip6_ACCESS-CM2_ssp245_heatDays_yearly"
        );
    }
}
