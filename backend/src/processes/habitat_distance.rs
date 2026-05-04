use crate::{
    db::{DbPool, PooledConnection},
    processes::parameters::PointGeoJsonInput,
};
use anyhow::{Context, Result};
use diesel::{
    sql_query,
    sql_types::{Double, Text},
};
use diesel_async::RunQueryDsl;
use geojson::PointType;
use ogcapi::{
    processes::Processor,
    types::{
        common::Link,
        processes::{
            Execute, ExecuteResult, ExecuteResults, InlineOrRefData, InputValueNoObject,
            JobControlOptions, Output, Process, ProcessSummary, TransmissionMode,
            description::{DescriptionType, InputDescription, OutputDescription},
        },
    },
};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;
use utoipa::ToSchema;

/// Calculates the Normalized Difference Vegetation Index (NDVI) and the corrected NDVI (kNDVI) from satellite imagery.
#[derive(Debug, Clone)]
pub struct HabitatDistanceProcess {
    connection: DbPool,
}

use diesel::QueryableByName;
use diesel::sql_types::Bool;

#[derive(QueryableByName)]
struct Natura2000Exists {
    #[diesel(sql_type = Bool)]
    exists: bool,
}

impl HabitatDistanceProcess {
    pub async fn new(connection: DbPool) -> Result<Self> {
        let this = Self { connection };

        let mut conn = this.connection().await?;
        let table: Natura2000Exists = sql_query(indoc::indoc! {"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'Natura2000'
                    AND table_name = 'naturasite_polygon'
            ) as exists
        "})
        .get_result(&mut *conn)
        .await
        .context("Failed to check if Natura2000.naturasite_polygon exists")?;

        if !table.exists {
            anyhow::bail!("Table Natura2000.naturasite_polygon does not exist");
        }

        // drop the pooled connection before moving `this` out
        drop(conn);

        Ok(this)
    }

    async fn connection(&self) -> anyhow::Result<PooledConnection<'_>> {
        self.connection
            .get()
            .await
            .context("could not get db connection from pool")
    }
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
pub struct HabitatDistanceProcessInputs {
    pub coordinate: PointGeoJsonInput,
}

#[derive(Deserialize, Serialize, Debug, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HabitatDistanceProcessOutputs {
    pub habitat_code: Option<String>,
    pub habitat_name: Option<String>,
    pub distance_m: Option<i64>,
}

impl From<HabitatDistanceProcessOutputs> for ExecuteResults {
    fn from(outputs: HabitatDistanceProcessOutputs) -> Self {
        let mut result = ExecuteResults::default();
        if let Some(habitat_code) = outputs.habitat_code {
            result.insert(
                "habitatCode".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        habitat_code,
                    )),
                },
            );
        }
        if let Some(habitat_name) = outputs.habitat_name {
            result.insert(
                "habitatName".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::String(
                        habitat_name,
                    )),
                },
            );
        }
        if let Some(distance_m) = outputs.distance_m {
            result.insert(
                "distanceM".to_string(),
                ExecuteResult {
                    output: Output {
                        format: None,
                        transmission_mode: Default::default(),
                    },
                    data: InlineOrRefData::InputValueNoObject(InputValueNoObject::Integer(
                        distance_m,
                    )),
                },
            );
        }
        result
    }
}

#[async_trait::async_trait]
impl Processor for HabitatDistanceProcess {
    fn id(&self) -> &'static str {
        "habitatDistance"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    #[allow(
        clippy::too_many_lines,
        reason = "description is long but better understood this way"
    )]
    fn process(&self) -> Result<Process> {
        let mut settings = SchemaSettings::default();
        settings.meta_schema = None;

        let mut generator = settings.into_generator();
        Ok(Process {
            summary: ProcessSummary {
                id: self.id().into(),
                version: self.version().into(),
                job_control_options: vec![
                    JobControlOptions::SyncExecute,
                    JobControlOptions::AsyncExecute,
                    // TODO: implement "dismiss extension"
                    // JobControlOptions::Dismiss,
                ],
                output_transmission: vec![TransmissionMode::Value],
                links: vec![
                    Link::new(
                        // TODO: ./ … does not work for some clients
                        format!("./{}/execution", self.id()),
                        "http://www.opengis.net/def/rel/ogc/1.0/execute",
                    )
                    .title("Execution endpoint"),
                ],
            },
            inputs: HashMap::from([(
                "coordinate".to_string(),
                InputDescription {
                    description_type: DescriptionType {
                        title: Some("Coordinate in WGS84".to_string()),
                        description: Some(
                            "This is a POINT input in WGS84 (EPSG:4326) format.".to_string(),
                        ),
                        ..Default::default()
                    },
                    schema: generator.root_schema_for::<PointGeoJsonInput>().to_value(),
                    ..Default::default()
                },
            )]),
            outputs: HashMap::from([
                (
                    "habitatCode".to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                "Habitat Code".to_string(),
                            ),
                            description: Some(
                                "This is the habitat code value of a Natura 2000 site. \
                                This is the state of 2024.".to_string(),
                            ),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<String>().to_value(),
                    },
                ),
                (
                    "habitatName".to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                "Habitat Name".to_string(),
                            ),
                            description: Some(
                                "This is the human-readable habitat name.".to_string(),
                            ),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<String>().to_value(),
                    },
                ),
                (
                    "habitatDistance".to_string(),
                    OutputDescription {
                        description_type: DescriptionType {
                            title: Some(
                                "Habitat Distance".to_string(),
                            ),
                            description: Some(
                                "This is the habitat distance value. \
                                The habitat distance is calculated based on the proximity to the nearest habitat of interest. \
                                The value is represented in meters.".to_string(),
                            ),
                            ..Default::default()
                        },
                        schema: generator.root_schema_for::<f64>().to_value(),
                    },
                ),
            ]),
        })
    }

    async fn execute(&self, execute: Execute) -> Result<ExecuteResults> {
        let value = serde_json::to_value(execute.inputs)?;
        let inputs: HabitatDistanceProcessInputs = serde_json::from_value(value)?;

        match compute_habitat_distance(
            self.connection().await?,
            &inputs.coordinate.value.coordinates,
        )
        .await
        {
            Ok(outputs) => Ok(outputs.into()),
            Err(_e) => Err(anyhow::anyhow!(
                "The server was unable to compute the habitat distance."
            )),
        }
    }
}

#[derive(QueryableByName)]
struct Natura2000NearestHabitat {
    #[diesel(sql_type = Text)]
    sitecode: String,
    #[diesel(sql_type = Text)]
    sitename: String,
    #[diesel(sql_type = Double)]
    distance_m: f64,
}

#[instrument(skip(connection), err(Debug))]
async fn compute_habitat_distance(
    mut connection: PooledConnection<'_>,
    coordinate: &PointType,
) -> Result<HabitatDistanceProcessOutputs> {
    let [lon, lat] = coordinate.as_slice() else {
        debug_assert!(false, "Expected PointType to have exactly 2 coordinates");
        return Err(anyhow::anyhow!("Invalid coordinate"));
    };
    let point_geometry = format!("SRID=4326;POINT({lon} {lat})");
    let table: Natura2000NearestHabitat = sql_query(indoc::indoc! {"
        WITH reference AS (
            SELECT ST_Transform($1::geometry, 3035) AS point
        )
        SELECT s.sitecode,
            s.sitename,
            ST_Distance(s.geom, reference.point) AS distance_m
        FROM \"Natura2000\".naturasite_polygon s, reference
        ORDER BY s.geom <-> reference.point
        LIMIT 1
    "})
    .bind::<Text, _>(point_geometry)
    .get_result(&mut *connection)
    .await
    .context("Failed to query Natura2000.naturasite_polygon")?;

    Ok(HabitatDistanceProcessOutputs {
        habitat_code: Some(table.sitecode),
        habitat_name: Some(table.sitename),
        distance_m: Some(table.distance_m.round() as i64),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CONFIG;
    use crate::db::setup_db;
    use diesel_async::SimpleAsyncConnection;
    use ogcapi::types::processes::Input;

    async fn mock_db_pool() -> DbPool {
        setup_db(&CONFIG.database).await.unwrap()
    }

    #[test]
    fn it_deserializes_the_input() {
        let json = serde_json::json!({
            "coordinate": {
                "value": {
                    "type": "Point",
                    "coordinates": [12.34, 56.78]
                },
                "mediaType": "application/geo+json"
            },
        });

        let inputs: HashMap<String, Input> = serde_json::from_value(json).unwrap();

        let json = serde_json::to_value(&inputs).unwrap();

        let _inputs: HabitatDistanceProcessInputs = serde_json::from_value(json).unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn compute_ndvi_integration_with_mock_backend() {
        let pool = mock_db_pool().await;

        // create schema / table and insert a test site
        let mut conn = pool.get().await.unwrap();
        conn.batch_execute(&indoc::formatdoc! {"
            CREATE SCHEMA IF NOT EXISTS \"Natura2000\";
            CREATE TABLE IF NOT EXISTS \"Natura2000\".naturasite_polygon (
                sitecode TEXT,
                sitename TEXT,
                geom geometry
            );

            INSERT INTO \"Natura2000\".naturasite_polygon (sitecode, sitename, geom)
            VALUES (
                'DE5417402',
                'Feldflur bei Hüttenberg und Schöffengrund',
                ST_GeomFromText('{wkt}', 3035)
            );
            ",
            wkt = include_str!("../../test-data/DE5417402.wkt"),
        })
        .await
        .unwrap();

        // consume the same connection in the computation (transaction stays open for test cleanup)
        let outputs = compute_habitat_distance(conn, &PointType::from((8.46, 50.49)))
            .await
            .unwrap();

        assert_eq!(outputs.habitat_code.unwrap(), "DE5417402");
        assert_eq!(
            outputs.habitat_name.unwrap(),
            "Feldflur bei Hüttenberg und Schöffengrund"
        );
        // distance should be very small (point exactly matches)
        assert_eq!(outputs.distance_m.unwrap(), 1415);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn process_summary_has_expected_inputs_and_outputs() {
        let pool = mock_db_pool().await;

        // create schema / table and insert a test site
        {
            let mut conn = pool.get().await.unwrap();
            conn.batch_execute(&indoc::formatdoc! {"
            CREATE SCHEMA IF NOT EXISTS \"Natura2000\";
            CREATE TABLE IF NOT EXISTS \"Natura2000\".naturasite_polygon (
                sitecode TEXT,
                sitename TEXT,
                geom geometry
            );
            "
            })
            .await
            .unwrap();
        }

        let p = HabitatDistanceProcess::new(pool).await.unwrap();
        let process = p.process().expect("to produce process description");

        // summary id / version
        assert_eq!(process.summary.id, "habitatDistance");
        assert_eq!(process.summary.version, "0.1.0");

        // job control options contain sync and async execute
        let mut has_sync = false;
        let mut has_async = false;
        for opt in &process.summary.job_control_options {
            match opt {
                JobControlOptions::SyncExecute => has_sync = true,
                JobControlOptions::AsyncExecute => has_async = true,
                JobControlOptions::Dismiss => todo!(),
            }
        }
        assert!(has_sync, "expected SyncExecute in job_control_options");
        assert!(has_async, "expected AsyncExecute in job_control_options");

        // inputs contain only coordinate
        assert!(process.inputs.contains_key("coordinate"));

        // outputs contain habitatCode, habitatName and habitatDistance
        assert!(process.outputs.contains_key("habitatCode"));
        assert!(process.outputs.contains_key("habitatName"));
        assert!(process.outputs.contains_key("habitatDistance"));

        // some basic checks for descriptions and schema presence
        let habitat_distance_output = &process.outputs["habitatDistance"];
        assert!(habitat_distance_output.schema.is_object());
    }
}
