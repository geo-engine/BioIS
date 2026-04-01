use crate::util::Secret;
use geoengine_api_client::apis::configuration::Configuration;
use std::{path::Path, sync::LazyLock};
use tracing::Level;
use tracing_subscriber::filter::Directive;
use url::Url;
use uuid::Uuid;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| get_config().expect("config can be loaded"));

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub geoengine: GeoEngineInstance,
    pub logging: Logging,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

impl From<&Server> for ogcapi::services::Config {
    fn from(server: &Server) -> Self {
        ogcapi::services::Config {
            host: server.host.clone(),
            port: server.port,
        }
    }
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
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct GeoEngineInstance {
    pub base_url: Url,
}

impl GeoEngineInstance {
    pub fn api_config(&self, session_token: Option<Secret<Uuid>>) -> Configuration {
        let mut configuration = Configuration::new();
        configuration.base_path = self.base_url.to_string();

        if let Some(session_token) = session_token {
            let session_token: Uuid = *session_token;
            configuration.bearer_access_token = Some(session_token.to_string());
        }

        configuration
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Logging {
    pub level: LogLevel,
    // pub target: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<Logging> for Directive {
    fn from(logging: Logging) -> Self {
        let level_filter = match logging.level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        };
        level_filter.into()
    }
}

fn get_config() -> anyhow::Result<Config> {
    let mut builder = config::Config::builder();

    builder = builder.add_source(config::File::from_str(
        include_str!("../conf/default.toml"),
        config::FileFormat::Toml,
    ));

    // TODO: other files
    let local_file = Path::new("Settings.toml");
    if local_file.exists() {
        builder = builder.add_source(config::File::from(local_file));
    }

    // TODO: environment variables

    Ok(builder.build()?.try_deserialize()?)
}
