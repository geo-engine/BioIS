use crate::util::Secret;
use geoengine_api_client::apis::configuration::Configuration;
use std::{path::Path, sync::LazyLock, time::Duration};
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
    pub data_ids: DataIdsConfig,
    pub credits: CreditsConfig,
    pub contact: ContactConfig,
    pub nexgddp_cmip6: NexGddpCmip6Config,
    pub logging: Logging,
}

/// Configuration of the climate-risk process, which derives its indicators from
/// NEX-GDDP-CMIP6 climate projections ingested as Geo Engine datasets.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct NexGddpCmip6Config {
    /// The model registry, listing the models and scenarios that have ingested datasets.
    /// Models are added by editing this file, without recompiling.
    pub model_registry_path: String,
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
    pub password: Secret<String>,
    pub clear_database_on_start: bool,
}

impl Database {
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{user}:{password}@{host}:{port}/{database}?options=-c%20search_path%3D{schema},public",
            user = self.user,
            password = self.password.expose(),
            host = self.host,
            port = self.port,
            database = self.database,
            schema = self.schema,
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

/// Data IDs configuration, containing identifiers for specific datasets used in the application.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct DataIdsConfig {
    pub land_use_imperviousness_builtup: String,
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

/// Specifies configuration for the credits system
#[derive(serde::Deserialize, Clone, Debug)]
pub struct CreditsConfig {
    pub credits_lookup_interval_seconds: u64,
    pub biodiversity_sensitive_areas: BiodiversitySensitiveAreasCreditsConfig,
    pub habitat_distance: HabitatDistanceCreditsConfig,
}

impl CreditsConfig {
    pub fn credits_lookup_interval(&self) -> Duration {
        Duration::from_secs(self.credits_lookup_interval_seconds)
    }
}

/// Specifies credits configuration for the biodiversity sensitive areas
#[derive(serde::Deserialize, Clone, Debug)]
pub struct BiodiversitySensitiveAreasCreditsConfig {
    pub credits_per_site: u64,
}

/// Specifies credits configuration for the habitat distance
#[derive(serde::Deserialize, Clone, Debug)]
pub struct HabitatDistanceCreditsConfig {
    pub credits_per_coordinate: u64,
}

/// Configuration for the contact form, which sends emails via SMTP.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ContactConfig {
    /// Sender of the notification emails, e.g. `BioIS <noreply@geoengine.de>`.
    pub from: String,
    /// Recipient of the notification emails.
    pub to: String,
    pub smtp: SmtpConfig,
    pub rate_limit: ContactRateLimitConfig,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub tls: SmtpTls,
    /// Name to use in the SMTP `EHLO` command. Defaults to the machine's hostname.
    pub helo_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<Secret<String>>,
}

#[derive(serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SmtpTls {
    /// Upgrade a plaintext connection via `STARTTLS` (usually port 587)
    Starttls,
    /// Implicit TLS (usually port 465)
    Tls,
    /// Unencrypted, only for local testing
    None,
}

/// Global rate limit for the contact form, shared by all clients.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ContactRateLimitConfig {
    pub max_requests: u64,
    pub window_seconds: u64,
}

impl ContactRateLimitConfig {
    pub fn window(&self) -> Duration {
        Duration::from_secs(self.window_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_info_log_level_to_directive() {
        let logging = Logging {
            level: LogLevel::Info,
        };

        let directive: Directive = logging.into();

        assert_eq!(directive.to_string(), "info");
    }
}
