use crate::{
    CONFIG,
    credits::add_credits_used,
    db::{DbHandle, model::ComputationId, util::RecordValueExt},
    processes::{parameters::PointGeoJsonInput, util::round_nearest_i64},
};
use anyhow::{Context, Result};
use geojson::PointType;
use indoc::formatdoc;
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
use toasty::{schema::db::Type as DbType, stmt::Type as StmtType};
use tracing::instrument;
use utoipa::ToSchema;

/// Calculates the distance to the nearest habitat of interest based on the provided coordinate input.
#[derive(Debug, Clone)]
pub struct HabitatDistanceProcess {
    connection: DbHandle,
    natura2000_schema: &'static str,
}

impl HabitatDistanceProcess {
    pub const ID: &'static str = "habitatDistance";

    pub async fn new(mut db: DbHandle, natura2000_schema: &'static str) -> Result<Self> {
        if !natura2000_exists(&mut db, natura2000_schema).await? {
            anyhow::bail!("Table {natura2000_schema}.naturasite_polygon does not exist");
        }

        Ok(Self {
            connection: db,
            natura2000_schema,
        })
    }
}

/// Check if the Natura2000 table exists
pub async fn natura2000_exists(db: &mut DbHandle, natura2000_schema: &'static str) -> Result<bool> {
    let result = toasty::sql::query(formatdoc! {"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = $1
                    AND table_name = 'naturasite_polygon'
            ) as exists
        "})
    .bind_typed(natura2000_schema, DbType::Text)
    .column_types([StmtType::Bool])
    .exec(db.as_mut())
    .await
    .context(format!(
        "Failed to check if {natura2000_schema}.naturasite_polygon exists"
    ))?
    .into_iter()
    .next()
    .context("No result returned from existence check")?;

    result.get_bool(0).context("Invalid exists type")
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
        Self::ID
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
                description: DescriptionType {
                    title: Some("Habitat Distance".to_string()),
                    description: Some(
                        "This process calculates the distance to the nearest habitat of interest based on the provided coordinate input."
                            .to_string(),
                    ),
                    ..Default::default()
                },
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
            &mut self.connection.clone(),
            self.natura2000_schema,
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

struct Natura2000NearestHabitat {
    sitecode: String,
    sitename: String,
    distance_m: f64,
}

impl TryFrom<toasty::stmt::Value> for Natura2000NearestHabitat {
    type Error = anyhow::Error;

    fn try_from(value: toasty::stmt::Value) -> Result<Self, Self::Error> {
        Ok(Natura2000NearestHabitat {
            sitecode: value.get_string(0).context("Invalid sitecode type")?,
            sitename: value.get_string(1).context("Invalid sitename type")?,
            distance_m: value.get_number(2).context("Invalid distance type")?,
        })
    }
}

#[instrument(skip(db), level = "debug", err(Debug))]
async fn compute_habitat_distance(
    db: &mut DbHandle,
    natura2000_schema: &str,
    coordinate: &PointType,
) -> Result<HabitatDistanceProcessOutputs> {
    let [lon, lat] = coordinate.as_slice() else {
        debug_assert!(false, "Expected PointType to have exactly 2 coordinates");
        return Err(anyhow::anyhow!("Invalid coordinate"));
    };
    let point_geometry = format!("SRID=4326;POINT({lon} {lat})");

    let table: Natura2000NearestHabitat = toasty::sql::query(formatdoc!(
        r#"WITH reference AS (
            SELECT ST_Transform($1::geometry, 3035) AS point
        )
        SELECT 
            s.sitecode,
            s.sitename,
            CAST(ST_Distance(s.geom, reference.point) AS DOUBLE PRECISION) AS distance_m
        FROM "{natura2000_schema}".naturasite_polygon s, reference
        ORDER BY s.geom <-> reference.point
        LIMIT 1"#
    ))
    .bind_typed(point_geometry, DbType::Text)
    .column_types([StmtType::String, StmtType::String, StmtType::F64])
    .exec(db.as_mut())
    .await
    .context(format!(
        "Failed to query {natura2000_schema}.naturasite_polygon"
    ))?
    .into_iter()
    .next()
    .context("No nearest habitat found for the given coordinate")?
    .try_into()?;

    add_credits_used(
        db.clone(),
        ComputationId::none(),
        CONFIG.credits.habitat_distance.credits_per_coordinate,
    )
    .await?;

    Ok(HabitatDistanceProcessOutputs {
        habitat_code: Some(table.sitecode),
        habitat_name: Some(table.sitename),
        distance_m: Some(round_nearest_i64(table.distance_m)),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::User,
        state::{CONTEXT, TaskLocalContext},
        util::Secret,
    };

    use super::*;
    use ogcapi::types::processes::Input;
    use uuid::Uuid;

    async fn create_schema_and_insert_test_site(db: &mut DbHandle) {
        let schema = db.schema_name().to_string();
        let wkt = include_str!("../../test-data/DE5417402.wkt");

        toasty::sql::statement(formatdoc! {r#"
            CREATE TABLE "{schema}".naturasite_polygon (
                sitecode TEXT,
                sitename TEXT,
                geom geometry
            )
        "#})
        .exec(db.as_mut())
        .await
        .unwrap();

        toasty::sql::statement(formatdoc!(
            r#"INSERT INTO "{schema}".naturasite_polygon (sitecode, sitename, geom)
            VALUES (
                'DE5417402',
                'Feldflur bei Hüttenberg und Schöffengrund',
                ST_GeomFromText($1, 3035)
            )"#,
        ))
        .bind_typed(wkt, DbType::Text)
        .exec(db.as_mut())
        .await
        .unwrap();
    }

    fn mock_user() -> User {
        User {
            id: Uuid::from_u128(0xabcd_efab_cdef_abcd_efab_cdef_abcd_efab),
            session_token: Secret(Uuid::from_u128(0x1234_5678_90ab_cdef_1234_5678_90ab_cdef)),
        }
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

    #[crate::test(task_context = crate::state::TaskContext::new(mock_user()))]
    async fn it_computes_the_nearest_habitat(mut db: DbHandle) {
        // crate::util::setup_tracing_for_tests();

        // create schema / table and insert a test site
        create_schema_and_insert_test_site(&mut db).await;

        CONTEXT
            .set_job_id(Uuid::from_u128(0x1234_5678_90ab_cdef_1234_5678_90ab_cdef))
            .unwrap();

        // compute the habitat distance
        let schema = db.schema_name().to_string();
        let outputs = compute_habitat_distance(&mut db, &schema, &PointType::from((8.46, 50.49)))
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

    #[crate::test]
    async fn process_summary_has_expected_inputs_and_outputs(mut db: DbHandle) {
        create_schema_and_insert_test_site(&mut db).await;

        let schema = db.schema_name().to_string();
        let p = HabitatDistanceProcess::new(db, schema.leak())
            .await
            .unwrap();
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
