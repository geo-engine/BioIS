use geoengine_api_client::models::{
    Aggregation, AggregationSum, ContinuousMeasurement, Expression, ExpressionParameters,
    MdGdalSource, MdGdalSourceParameters, Measurement, RasterBandDescriptor, RasterOperator,
    SingleRasterSource, TemporalRasterAggregation, TemporalRasterAggregationParameters,
    TimeGranularity, TimeStep,
};
use tracing::instrument;

use super::{ClimateRiskProcess, types::*};
impl ClimateRiskProcess {
    /// Builds the netCDF raster source for a variable/model/scenario combination.
    #[instrument(skip(var, model, scenario))]
    pub(crate) fn dataset_raster_source(
        var: &ClimateVariableProperties,
        model: &ClimateModelProperties,
        scenario: &ClimateScenarioProperties,
    ) -> RasterOperator {
        let dataset_name = format!(
            "nexgddp_cmip6_{}_{}_{}",
            model.id, scenario.dataset_prefix, var.dataset_variable_suffix
        );
        // ponytail: `GdalSource` cannot read an Md dataset — its `GdalMetaData` enum has no MD
        // variant, so the registered netCDF files are only reachable through `MdGdalSource`.
        RasterOperator::MdGdalSource(
            MdGdalSource {
                r#type: Default::default(),
                params: MdGdalSourceParameters { data: dataset_name }.into(),
            }
            .into(),
        )
    }

    /// Builds the daily threshold expression (e.g. heat-day indicator) for a variable.
    pub(crate) fn build_variable_day_expression(
        var: &ClimateVariableProperties,
        model: &ClimateModelProperties,
        scenario: &ClimateScenarioProperties,
    ) -> RasterOperator {
        RasterOperator::Expression(
            Expression {
                r#type: Default::default(),
                params: ExpressionParameters {
                    expression: var.expression_string(),
                    output_type: ClimateVariableProperties::expression_dtype(),
                    output_band: Some(
                        RasterBandDescriptor {
                            name: var.name_string(),
                            measurement: Measurement::Continuous(
                                ContinuousMeasurement {
                                    measurement: ClimateVariableProperties::measurement_string(),
                                    r#type: Default::default(),
                                    unit: Some(Some(ClimateVariableProperties::measurement_unit())),
                                }
                                .into(),
                            )
                            .into(),
                        }
                        .into(),
                    ),
                    map_no_data: false,
                }
                .into(),
                sources: SingleRasterSource {
                    raster: Self::dataset_raster_source(var, model, scenario).into(),
                }
                .into(),
            }
            .into(),
        )
    }

    /// Builds the yearly sum aggregation workflow (sums daily values per calendar year).
    pub(crate) fn build_variable_year_agg_workflow(
        var: &ClimateVariableProperties,
        model: &ClimateModelProperties,
        scenario: &ClimateScenarioProperties,
    ) -> RasterOperator {
        RasterOperator::TemporalRasterAggregation(
            TemporalRasterAggregation {
                r#type: Default::default(),
                params: TemporalRasterAggregationParameters {
                    aggregation: Aggregation::Sum(Box::new(AggregationSum {
                        ignore_no_data: true,
                        r#type: Default::default(),
                    }))
                    .into(),
                    output_type: Some(Some(ClimateVariableProperties::year_agg_dtype())),
                    window: TimeStep {
                        granularity: TimeGranularity::Years,
                        step: 1,
                    }
                    .into(),
                    // No reference: yearly windows are anchored at the epoch, i.e. calendar-aligned,
                    // so one workflow serves both the analysis and the reference time range.
                    window_reference: None,
                }
                .into(),
                sources: SingleRasterSource {
                    raster: Self::build_variable_day_expression(var, model, scenario).into(),
                }
                .into(),
            }
            .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_variable_workflow_chains() {
        let scenario = ClimateScenario::Ssp245.properties();
        let model = ClimateModelProperties {
            id: "ACCESS-CM2".to_string(),
            variant: "r1i1p1f1".to_string(),
            grid: "gn".to_string(),
            scenarios: vec![ClimateScenario::Historical, ClimateScenario::Ssp245],
        };
        let var = ClimateVariable::HeatDays.properties();

        let day_expr = ClimateRiskProcess::build_variable_day_expression(&var, &model, &scenario);
        assert!(matches!(day_expr, RasterOperator::Expression(_)));

        let year_agg =
            ClimateRiskProcess::build_variable_year_agg_workflow(&var, &model, &scenario);
        assert!(matches!(
            year_agg,
            RasterOperator::TemporalRasterAggregation(_)
        ));
    }
}
