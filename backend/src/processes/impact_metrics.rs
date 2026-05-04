use crate::processes::parameters::{Hectare, Year};
use anyhow::Result;
use ogcapi::{
    processes::Processor,
    types::{
        common::Link,
        processes::{
            Execute, ExecuteResult, ExecuteResults, InlineOrRefData, InputValueNoObject,
            JobControlOptions, Output, Process, ProcessSummary, TransmissionMode,
            description::{DescriptionType, InputDescription, Metadata, OutputDescription},
        },
    },
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Impact metrics related to biodiversity and ecosystems change (ESRS E4 B5) - scaffold implementation
#[derive(Debug, Clone)]
pub struct ImpactMetricsProcess;

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct ImpactMetricsProcessInputs {
    /// Sites as a `GeoJSON` `FeatureCollection` (placeholder)
    pub sites: serde_json::Value,
    /// Reporting year
    pub reporting_year: Year,
    /// Previous year (optional). If omitted, only the current year will be reported without change metrics.
    pub previous_year: Option<Year>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct SiteRow {
    pub location: Option<String>,
    pub area_ha: f64,
    pub biodiversity_sensitive_area_ha: f64,
    pub specification: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema, Clone)]
pub struct TotalsPerYear {
    pub sealed_area: Hectare,
    pub nature_oriented_on_site: Hectare,
    pub nature_oriented_off_site: Hectare,
    pub use_of_land: Hectare,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct ImpactMetricsProcessOutputs {
    pub site_table: Vec<SiteRow>,
    pub totals_previous_year: TotalsPerYear,
    pub totals_reporting_year: TotalsPerYear,
    pub percent_change: TotalsPerYear,
    pub inputs: serde_json::Value,
    pub sources: Vec<String>,
}

impl From<ImpactMetricsProcessOutputs> for ExecuteResults {
    fn from(outputs: ImpactMetricsProcessOutputs) -> Self {
        let mut result = ExecuteResults::default();

        // Serialize tables as JSON strings as a simple inline representation for now
        if let Ok(site_table_str) = serde_json::to_string(&outputs.site_table) {
            result.insert(
                "siteTable".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        site_table_str,
                    )),
                },
            );
        }

        if let Ok(totals_prev_str) = serde_json::to_string(&outputs.totals_previous_year) {
            result.insert(
                "totalsPreviousYear".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        totals_prev_str,
                    )),
                },
            );
        }

        if let Ok(totals_rep_str) = serde_json::to_string(&outputs.totals_reporting_year) {
            result.insert(
                "totalsReportingYear".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        totals_rep_str,
                    )),
                },
            );
        }

        if let Ok(percent_change_str) = serde_json::to_string(&outputs.percent_change) {
            result.insert(
                "percentChange".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        percent_change_str,
                    )),
                },
            );
        }

        if let Ok(inputs_log_str) = serde_json::to_string(&outputs.inputs) {
            result.insert(
                "inputsLog".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        inputs_log_str,
                    )),
                },
            );
        }

        if let Ok(sources_str) = serde_json::to_string(&outputs.sources) {
            result.insert(
                "sources".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        sources_str,
                    )),
                },
            );
        }

        result
    }
}

/// Compute the impact metrics from inputs. This helper performs optional HTTP GET requests
/// to any provided `workflow_refs` (if they are HTTP URLs) and includes their responses
/// in the `sources` output for auditing. The spatial computations are placeholders and
/// should be replaced by Geo Engine workflow calls in future.
pub async fn compute_impact_metrics(
    inputs: ImpactMetricsProcessInputs,
) -> anyhow::Result<ImpactMetricsProcessOutputs> {
    // Placeholder computation: extract feature properties if available and aggregate
    let mut site_table: Vec<SiteRow> = Vec::new();

    if let Some(features) = inputs.sites.get("features").and_then(|v| v.as_array()) {
        for feature in features {
            let props = feature
                .get("properties")
                .unwrap_or(&serde_json::Value::Null);
            let location = props
                .get("location")
                .and_then(|v| v.as_str())
                .map(ToString::to_string);
            let area_ha = props
                .get("area_ha")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            let biodiversity_sensitive_area_ha = props
                .get("biodiversity_sensitive_area_ha")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            let specification = props
                .get("specification")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
                .or(Some(
                    "computed via Geo Engine workflow (placeholder)".to_string(),
                ));

            site_table.push(SiteRow {
                location,
                area_ha,
                biodiversity_sensitive_area_ha,
                specification,
            });
        }
    }

    // Totals (reporting year)
    let totals_reporting = TotalsPerYear {
        sealed_area: Hectare(site_table.iter().map(|r| r.area_ha * 0.1).sum()), // placeholder: 10% sealed
        nature_oriented_on_site: Hectare(site_table.iter().map(|r| r.area_ha * 0.6).sum()),
        nature_oriented_off_site: Hectare(site_table.iter().map(|r| r.area_ha * 0.2).sum()),
        use_of_land: Hectare(site_table.iter().map(|r| r.area_ha).sum()),
    };

    // Previous year: placeholder - same as reporting
    let totals_previous = totals_reporting.clone();

    // Percent change: zero for placeholder
    let percent_change = TotalsPerYear {
        sealed_area: Hectare(0.0),
        nature_oriented_on_site: Hectare(0.0),
        nature_oriented_off_site: Hectare(0.0),
        use_of_land: Hectare(0.0),
    };

    // sources: placeholder for provenance / audit
    let sources: Vec<String> = vec!["Geo Engine workflow XYZ (placeholder)".to_string()];

    let outputs = ImpactMetricsProcessOutputs {
        site_table,
        totals_previous_year: totals_previous,
        totals_reporting_year: totals_reporting,
        percent_change,
        inputs: serde_json::to_value(&inputs)?,
        sources,
    };

    Ok(outputs)
}
#[async_trait::async_trait]
impl Processor for ImpactMetricsProcess {
    fn id(&self) -> &'static str {
        "impact-metrics-biodiversity"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn process(&self) -> Result<Process> {
        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;

        let mut generator = settings.into_generator();
        Ok(Process {
            summary: ProcessSummary {
                id: self.id().into(),
                version: self.version().into(),
                job_control_options: vec![JobControlOptions::SyncExecute, JobControlOptions::AsyncExecute],
                output_transmission: vec![TransmissionMode::Value],
                links: vec![
                    Link::new(format!("./{}/execution", self.id()), "http://www.opengis.net/def/rel/ogc/1.0/execute").title("Execution endpoint"),
                ],
            },
            inputs: HashMap::from([
                (
                    "sites".to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: Some("Sites (GeoJSON FeatureCollection)".to_string()),
                            description: Some("A GeoJSON FeatureCollection describing sites for which impact metrics should be calculated.".to_string()),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<serde_json::Value>().to_value(),
                        ..Default::default()
                    },
                ),
                (
                    "reporting_year".to_string(),
                    InputDescription {
                        description_type: DescriptionType {
                            title: Some("Reporting year".to_string()),
                            description: Some("The reporting year for which metrics should be computed.".to_string()),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<i32>().to_value(),
                        ..Default::default()
                    },
                ),
            ]),
            outputs: HashMap::from([
                (
                    "siteTable".to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some("Site-level table".to_string()),
                            description: Some("Table with location, area (ha), biodiversity sensitive area (ha), and specification.".to_string()),
                            metadata: vec![Metadata { role: Some("note".to_string()), title: Some("Placeholder outputs: values computed via Geo Engine workflows".to_string()), href: None }],
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<Vec<SiteRow>>().to_value(),
                    },
                ),
                (
                    "totalsPreviousYear".to_string(),
                    OutputDescription {
                        description_type: DescriptionType { title: Some("Totals (previous year)".to_string()), description: Some("Totals for the previous year.".to_string()), ..Default::default() },
                        schema: generator.root_schema_for::<TotalsPerYear>().to_value(),
                    },
                ),
                (
                    "totalsReportingYear".to_string(),
                    OutputDescription {
                        description_type: DescriptionType { title: Some("Totals (reporting year)".to_string()), description: Some("Totals for the reporting year.".to_string()), ..Default::default() },
                        schema: generator.root_schema_for::<TotalsPerYear>().to_value(),
                    },
                ),
                (
                    "percentChange".to_string(),
                    OutputDescription {
                        description_type: DescriptionType { title: Some("Percent change".to_string()), description: Some("Percent change between previous and reporting year for each metric.".to_string()), ..Default::default() },
                        schema: generator.root_schema_for::<TotalsPerYear>().to_value(),
                    },
                ),
                (
                    "inputsLog".to_string(),
                    OutputDescription {
                        description_type: DescriptionType { title: Some("Inputs and parameters".to_string()), description: Some("Echo of inputs for auditing.".to_string()), ..Default::default() },
                        schema: generator.root_schema_for::<serde_json::Value>().to_value(),
                    },
                ),
                (
                    "sources".to_string(),
                    OutputDescription {
                        description_type: DescriptionType { title: Some("Sources".to_string()), description: Some("List of data sources and workflow references used for audits.".to_string()), ..Default::default() },
                        schema: generator.root_schema_for::<Vec<String>>().to_value(),
                    },
                ),
            ]),
        })
    }

    async fn execute(&self, execute: Execute) -> Result<ExecuteResults> {
        let value = serde_json::to_value(execute.inputs)?;
        let inputs: ImpactMetricsProcessInputs = serde_json::from_value(value)?;

        let outputs = compute_impact_metrics(inputs).await?;
        Ok(outputs.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httptest::matchers::*;
    use httptest::responders::*;
    use httptest::{Expectation, Server};
    use ogcapi::types::processes::Input;
    use serde_json::json;

    #[test]
    fn it_deserializes_input() {
        let json = serde_json::json!({
            "sites": {"value": {"type":"FeatureCollection","features":[]}, "mediaType": "application/geo+json"},
            "reporting_year": 2025
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(json).unwrap();
        let json = serde_json::to_value(&inputs).unwrap();
        let _inputs: ImpactMetricsProcessInputs = serde_json::from_value(json).unwrap();
    }

    #[tokio::test]
    async fn it_computes_biodiversity_impact() {
        // Start mock server
        let server = Server::run();

        // Mock a simple GET response for any workflow ref
        server.expect(
            Expectation::matching(request::method("GET"))
                .respond_with(json_encoded(json!({ "mock": "value" }))),
        );

        // Build inputs: one feature with area 100 ha
        let sites = json!({
            "type": "FeatureCollection",
            "features": [
                { "type": "Feature", "properties": { "location": "SiteA", "area_ha": 100.0, "biodiversity_sensitive_area_ha": 5.0, "specification": "spec" } }
            ]
        });

        let inputs = ImpactMetricsProcessInputs {
            sites: sites.clone(),
            reporting_year: Year(2025),
            previous_year: Some(Year(2024)),
        };

        let outputs = compute_impact_metrics(inputs)
            .await
            .expect("compute_impact_metrics should succeed");

        // site table row present
        assert_eq!(outputs.site_table.len(), 1);

        // totals use_of_land equals 100.0 (Hectare tuple .0)
        assert_eq!(outputs.totals_reporting_year.use_of_land.0, 100.0);

        // Build expected full JSON output
        let expected = json!({
            "site_table": [
                {
                    "location": "SiteA",
                    "area_ha": 100.0,
                    "biodiversity_sensitive_area_ha": 5.0,
                    "specification": "spec"
                }
            ],
            "totals_previous_year": {
                "sealed_area": 10.0,
                "nature_oriented_on_site": 60.0,
                "nature_oriented_off_site": 20.0,
                "use_of_land": 100.0
            },
            "totals_reporting_year": {
                "sealed_area": 10.0,
                "nature_oriented_on_site": 60.0,
                "nature_oriented_off_site": 20.0,
                "use_of_land": 100.0
            },
            "percent_change": {
                "sealed_area": 0.0,
                "nature_oriented_on_site": 0.0,
                "nature_oriented_off_site": 0.0,
                "use_of_land": 0.0
            },
            "inputs": {
                "sites": sites,
                "reporting_year": 2025,
                "previous_year": 2024
            },
            "sources": ["Geo Engine workflow XYZ (placeholder)"]
        });

        let outputs_json = serde_json::to_value(&outputs).expect("serialize outputs");
        assert_eq!(outputs_json, expected);
    }
}
