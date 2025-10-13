use anyhow::Result;
use ogcapi::{
    processes::{ProcessResponseBody, Processor},
    types::processes::{Execute, Process},
};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Echo;

#[derive(Deserialize, Debug, JsonSchema)]
pub struct EchoInputs {
    pub string_input: Option<String>,
    pub measure_input: Option<MeasureInput>,
    pub date_input: Option<String>,
    pub double_input: Option<f64>,
    pub array_input: Option<Vec<i32>>,
    pub complex_object_input: Option<ComplexObjectInput>,
    pub geometry_input: Option<Vec<String>>,
    pub bounding_box_input: Option<BoundingBoxInput>,
    pub images_input: Option<Vec<String>>,
    pub feature_collection_input: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
pub struct MeasureInput {
    pub measurement: f64,
    pub uom: String,
    pub reference: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
pub struct ComplexObjectInput {
    pub property1: String,
    pub property2: Option<String>,
    pub property3: Option<f64>,
    pub property4: Option<String>,
    pub property5: bool,
}

#[derive(Clone, Deserialize, Serialize, Debug, JsonSchema)]
pub struct BoundingBoxInput {
    pub bbox: Vec<f64>,
}

// impl EchoInputs {
//     pub fn execute_input(&self) -> HashMap<String, Input> {
//         HashMap::from([(
//             "name".to_string(),
//             Input::InlineOrRefData(InlineOrRefData::InputValueNoObject(
//                 InputValueNoObject::String(self.name.to_owned()),
//             )),
//         )])
//     }
// }

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct EchoOutputs {
    pub string_input: Option<String>,
    pub measure_input: Option<MeasureInput>,
    pub date_input: Option<String>,
    pub double_input: Option<f64>,
    pub array_input: Option<Vec<i32>>,
    pub complex_object_input: Option<ComplexObjectInput>,
    pub geometry_input: Option<Vec<String>>,
    pub bounding_box_input: Option<BoundingBoxInput>,
    pub images_input: Option<Vec<String>>,
    pub feature_collection_input: Option<String>,
}

// impl EchoOutputs {
//     pub fn execute_output() -> HashMap<String, Output> {
//         HashMap::from([(
//             "greeting".to_string(),
//             Output {
//                 format: Some(Format {
//                     media_type: Some("text/plain".to_string()),
//                     encoding: Some("utf8".to_string()),
//                     schema: None,
//                 }),
//                 transmission_mode: TransmissionMode::Value,
//             },
//         )])
//     }
// }

// impl TryFrom<ProcessResponseBody> for EchoOutputs {
//     type Error = Exception;

//     fn try_from(value: ProcessResponseBody) -> Result<Self, Self::Error> {
//         if let ProcessResponseBody::Requested(buf) = value {
//             Ok(EchoOutputs {
//                 greeting: String::from_utf8(buf).unwrap(),
//             })
//         } else {
//             Err(Exception::new("500"))
//         }
//     }
// }

#[async_trait::async_trait]
impl Processor for Echo {
    fn id(&self) -> &'static str {
        "echo"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn process(&self) -> Result<Process> {
        Process::try_new(
            self.id(),
            self.version(),
            &schema_for!(EchoInputs),
            &schema_for!(EchoOutputs),
        )
        .map_err(Into::into)
    }

    async fn execute(&self, execute: Execute) -> Result<ProcessResponseBody> {
        let value = serde_json::to_value(execute.inputs).unwrap();
        let inputs: EchoInputs = serde_json::from_value(value).unwrap();

        let outputs = EchoOutputs {
            string_input: inputs.string_input,
            measure_input: inputs.measure_input,
            date_input: inputs.date_input,
            double_input: inputs.double_input,
            array_input: inputs.array_input,
            complex_object_input: inputs.complex_object_input,
            geometry_input: inputs.geometry_input,
            bounding_box_input: inputs.bounding_box_input,
            images_input: inputs.images_input,
            feature_collection_input: inputs.feature_collection_input,
        };

        let response = serde_json::to_vec(&outputs)?;

        Ok(ProcessResponseBody::Requested(response))
    }
}
