#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "This is a binary for fetching the OpenAPI spec, so printing to stdout/stderr is fine"
)]

use anyhow::Context;
use reqwest::IntoUrl;
use std::process::Stdio;
use tokio::time::{Duration, sleep};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = Server::start("http://localhost:4040")?;

    // Wait for the server to become available and fetch the OpenAPI spec.
    let (mut sleep_duration, backoff_factor) = (Duration::from_millis(200), 1.5);
    let mut openapi_spec = None;
    for _ in 0..10 {
        eprintln!("Trying to fetch OpenAPI spec from server…");
        if let Ok(spec) = server.try_fetch_openapi_spec().await {
            openapi_spec = Some(spec);
            break;
        }
        sleep(sleep_duration).await;
        sleep_duration = sleep_duration.mul_f64(backoff_factor);
    }

    let openapi_spec =
        openapi_spec.context("Failed to fetch OpenAPI spec from server after multiple attempts")?;

    // Write the OpenAPI spec to stdout.
    println!("{}", serde_json::to_string_pretty(&openapi_spec)?);

    server.shutdown().await
}

struct Server {
    child: tokio::process::Child,
    base_url: Url,
}

// const OPEN_API_PATH: &str = "api";
const OPEN_API_PATH: &str = "api_v3.1";

impl Server {
    /// Starts the `BioIS` server by running `cargo run`.
    fn start<U: IntoUrl>(base_url: U) -> anyhow::Result<Self> {
        // Spawn the server as a child process.
        let child = tokio::process::Command::new("cargo")
            .arg("run")
            .stdout(Stdio::null())
            .spawn()
            .context("Failed to spawn geoengine server")?;

        Ok(Self {
            child,
            base_url: base_url.into_url()?,
        })
    }

    #[cfg(test)]
    fn start_for_testing<U: IntoUrl>(base_url: U) -> anyhow::Result<Self> {
        Ok(Self {
            child: tokio::process::Command::new("true")
                .stdout(Stdio::null())
                .spawn()
                .context("Failed to spawn dummy server process")?,
            base_url: base_url.into_url()?,
        })
    }

    async fn try_fetch_openapi_spec(&self) -> anyhow::Result<serde_json::Value> {
        let api_url = self.base_url.join(OPEN_API_PATH)?;

        reqwest::get(api_url.clone())
            .await
            .with_context(|| format!("Failed to GET OpenAPI spec from {api_url}"))?
            .error_for_status()
            .with_context(|| format!("Non-success response from {api_url}"))?
            .json()
            .await
            .context("Failed to read response body as json")
    }

    /// Shuts down the server process.
    async fn shutdown(mut self) -> anyhow::Result<()> {
        self.child
            .kill()
            .await
            .context("Failed to kill geoengine server process")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{Expectation, Server as MockServer, matchers::*, responders::status_code};

    #[tokio::test]
    async fn it_fetches_openapi_spec_successfully() {
        let server = MockServer::run();
        let body = serde_json::json!({"openapi": "3.0.0"});

        server.expect(
            Expectation::matching(request::method_path("GET", "/api"))
                .respond_with(status_code(200).body(body.to_string())),
        );

        let srv = Server::start_for_testing(server.url("/").to_string()).unwrap();

        let json = srv
            .try_fetch_openapi_spec()
            .await
            .expect("should fetch json");

        assert_eq!(json["openapi"], "3.0.0");
    }
}
