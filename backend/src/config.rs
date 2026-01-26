use anyhow::Context;
use geoengine_openapi_client::apis::configuration::Configuration;
use std::sync::LazyLock;
use url::Url;

use crate::auth::User;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| get_config().expect("config can be loaded"));

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub geoengine: GeoEngineInstance,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Database {
    pub host: String,
    pub port: u16,
    #[allow(clippy::struct_field_names)]
    pub database: String,
    pub schema: String,
    pub user: String,
    pub password: String,
    pub clear_database_on_start: bool,
}

impl Database {
    pub fn connection_string(&self) -> anyhow::Result<Url> {
        Url::parse(&format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        ))
        .context("failed to parse database connection string")
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct GeoEngineInstance {
    pub base_url: Url,
}

impl GeoEngineInstance {
    pub fn api_config(&self, user_session: Option<&User>) -> Configuration {
        let mut configuration = Configuration::new();
        configuration.base_path = self.base_url.to_string();

        if let Some(user) = user_session {
            configuration.bearer_access_token = Some(user.session_token.to_string());
        }

        configuration
    }
}

fn get_config() -> anyhow::Result<Config> {
    let mut builder = config::Config::builder();

    builder = builder.add_source(config::File::from_str(
        include_str!("../conf/default.toml"),
        config::FileFormat::Toml,
    ));

    // TODO: other files

    // TODO: environment variables

    Ok(builder.build()?.try_deserialize()?)
}
