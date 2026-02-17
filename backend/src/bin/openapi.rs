#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "This is a binary for fetching the OpenAPI spec, so printing to stdout/stderr is fine"
)]

use anyhow::Context;
use biois::server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", openapi_json().await?);

    Ok(())
}

async fn openapi_json() -> anyhow::Result<String> {
    let mut service = server().await?;

    service
        .get_router_mut()
        .get_openapi_mut()
        .to_pretty_json()
        .context("Failed to serialize OpenAPI spec as JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn it_fetches_openapi_spec_successfully() {
        let openapi_json: serde_json::Value =
            serde_json::from_str(&openapi_json().await.unwrap()).unwrap();

        assert_eq!(openapi_json["openapi"], "3.1.0");
        assert_eq!(openapi_json["info"]["title"], "BioIS API");
    }
}
