use anyhow::{Context, Result};
use biois::{CONFIG, db::DbHandle, server, setup_tracing};
use clap::Parser;
use std::ffi::OsString;
use toasty_cli::{Config, ToastyCli};

#[tokio::main]
async fn main() -> Result<()> {
    let toasty_config = Config::load()?;

    let db = DbHandle::from_config(&CONFIG.database).await?;

    let toasty_cli = ToastyCli::with_config(db.db(), toasty_config);

    match Cli::parse().command {
        Command::Database(cmd) => cmd.run(&toasty_cli).await?,
        Command::OpenAPI(cmd) => cmd.run().await?,
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(name = "BioIS CLI")]
#[command(about = "BioIS CLI - BioIS Management Tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Database migration commands
    Database(DatabaseCommand),

    #[command(name = "openapi")]
    OpenAPI(OpenAPICommand),
}

#[derive(Parser, Debug)]
struct DatabaseCommand {
    #[arg()]
    args: Vec<OsString>,
}

impl DatabaseCommand {
    async fn run(self, toasty_cli: &ToastyCli) -> Result<()> {
        let args = std::iter::once(OsString::from("toasty")).chain(self.args.into_iter());
        toasty_cli.parse_from(args).await
    }
}

#[derive(Parser, Debug)]
struct OpenAPICommand;

impl OpenAPICommand {
    async fn run(self) -> Result<()> {
        let _tracing_guard = setup_tracing(CONFIG.logging.clone().into());
        println!("{}", openapi_json().await?);

        Ok(())
    }
}

async fn openapi_json() -> Result<String> {
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
