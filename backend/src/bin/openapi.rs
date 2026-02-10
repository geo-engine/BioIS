#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "This is a binary for fetching the OpenAPI spec, so printing to stdout/stderr is fine"
)]

use anyhow::Context;
use reqwest::IntoUrl;
use std::process::Stdio;
use tokio::{
    process::ChildStderr,
    time::{Duration, sleep},
};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let mut server = Server::start("http://localhost:4040")?;

    check_for_start_log(server.stderr()).await?;

    // Wait for the server to become available and fetch the OpenAPI spec.
    let (mut sleep_duration, backoff_factor) = (Duration::from_millis(1_000), 1.5);
    let mut openapi_spec = None;
    for _ in 0..10 {
        info!("Trying to fetch OpenAPI spec from server…");
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

/// Checks the server's stderr for the startup log message, and waits until it is found.
async fn check_for_start_log(stderr: ChildStderr) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut reader = BufReader::new(stderr).lines();

    while let Some(line) = reader.next_line().await? {
        info!(target: "server", line);
        if line.contains("Running") && line.contains("BioIS") {
            return Ok(());
        }
    }

    Err(anyhow::anyhow!(
        "Server process exited before startup log message was found"
    ))
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_writer(std::io::stderr),
        )
        .init();
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
            .stderr(Stdio::piped())
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

    fn stderr(&mut self) -> ChildStderr {
        self.child.stderr.take().expect("failed to read stderr")
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
            Expectation::matching(request::method_path("GET", format!("/{OPEN_API_PATH}")))
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
