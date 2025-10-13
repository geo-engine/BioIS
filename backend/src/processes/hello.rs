use anyhow::Result;
use ogcapi::{
    processes::{ProcessResponseBody, Processor},
    types::processes::{Execute, Process},
};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct HelloProcess;

#[derive(Deserialize, Debug, JsonSchema)]
pub struct HelloInputs {
    pub name: String,
}

#[derive(Serialize, Clone, Debug, JsonSchema)]
pub struct HelloOutputs {
    pub sentence: String,
}

#[async_trait::async_trait]
impl Processor for HelloProcess {
    fn id(&self) -> &'static str {
        "hello"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn process(&self) -> Result<Process> {
        Process::try_new(
            self.id(),
            self.version(),
            &schema_for!(HelloInputs),
            &schema_for!(HelloOutputs),
        )
        .map_err(Into::into)
    }

    async fn execute(&self, execute: Execute) -> Result<ProcessResponseBody> {
        let value = serde_json::to_value(execute.inputs)?;
        let inputs: HelloInputs = serde_json::from_value(value)?;
        let output = HelloOutputs {
            sentence: format!("Hello, {}!", inputs.name),
        };
        Ok(ProcessResponseBody::Requested(serde_json::to_vec_pretty(
            &output,
        )?))
    }
}
