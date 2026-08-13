use geoengine_api_client::models::{
    Aggregation, ContinuousMeasurement, Expression, ExpressionParameters, GdalSource,
    GdalSourceParameters, Measurement, RasterBandDescriptor, RasterOperator, SingleRasterSource,
    SumAggregation, TemporalRasterAggregation, TemporalRasterAggregationParameters,
    TimeGranularity, TimeStep,
};
use tracing::instrument;

use super::{ClimateRiskProcess, types::*};
impl ClimateRiskProcess {
    #[instrument(skip(var, model, scenario, region))]
    pub(crate) fn dataset_raster_source(
        var: &ClimateVariableProperties,
        model: &CordexModelProperties,
        scenario: &ClimateScenarioProperties,
        region: &CordexRegionProperties,
    ) -> RasterOperator {
        let dataset_name = format!(
            "cordex_{}_{}_{}_{}",
            region.dataset_prefix,
            scenario.dataset_prefix,
            model.dataset_prefix,
            var.dataset_variable_suffix
        );
        RasterOperator::GdalSource(
            GdalSource {
                r#type: Default::default(),
                params: GdalSourceParameters {
                    data: dataset_name,
                    overview_level: None,
                }
                .into(),
            }
            .into(),
        )
    }

    pub(crate) fn build_variable_day_expression(
        var: &ClimateVariableProperties,
        model: &CordexModelProperties,
        scenario: &ClimateScenarioProperties,
        region: &CordexRegionProperties,
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
                    raster: Self::dataset_raster_source(var, model, scenario, region).into(),
                }
                .into(),
            }
            .into(),
        )
    }

    pub(crate) fn build_variable_year_agg_workflow(
        var: &ClimateVariableProperties,
        model: &CordexModelProperties,
        scenario: &ClimateScenarioProperties,
        region: &CordexRegionProperties,
    ) -> RasterOperator {
        RasterOperator::TemporalRasterAggregation(
            TemporalRasterAggregation {
                r#type: Default::default(),
                params: TemporalRasterAggregationParameters {
                    aggregation: Aggregation::SumAggregation(Box::new(SumAggregation {
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
                    raster: Self::build_variable_day_expression(var, model, scenario, region)
                        .into(),
                }
                .into(),
            }
            .into(),
        )
    }
}
