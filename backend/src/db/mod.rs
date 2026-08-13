use anyhow::{Context, Result};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};
use tracing::{info, instrument};

pub mod model;
#[cfg(test)]
pub mod tests;
pub mod util;

/// A handle to a database connection, including the schema name.
#[derive(Clone, Debug)]
pub struct DbHandle {
    db: toasty::Db,
    #[allow(unused, reason = "used at least in tests")]
    schema: Arc<str>,
}

impl Deref for DbHandle {
    type Target = toasty::Db;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl DerefMut for DbHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl AsMut<toasty::Db> for DbHandle {
    fn as_mut(&mut self) -> &mut toasty::Db {
        &mut self.db
    }
}

impl DbHandle {
    const MIGRATIONS: toasty::migration::MigrationSet = toasty::embed_migrations!("database");

    /// Creates a new database handle from the given configuration.
    pub async fn from_config(config: &crate::config::Database) -> Result<Self> {
        let db = toasty::Db::builder()
            .models(toasty::models!(model::Job))
            .max_pool_size(if cfg!(test) { 1 } else { 8 })
            .pool_wait_timeout(Some(Duration::from_secs(5)))
            .pool_create_timeout(Some(Duration::from_secs(10)))
            .pool_health_check_interval(Some(Duration::from_mins(1)))
            .connect(&config.connection_string())
            .await
            .context("Failed to connect to database")?;

        let mut this = Self {
            db,
            schema: Arc::from(config.schema.as_str()),
        };
        this.setup(config).await?;

        Ok(this)
    }

    /// Initializes the database schema, dropping and recreating it if `clear_database_on_start` is true.
    ///
    /// On tests, the schema is always dropped and recreated to ensure a clean state.
    #[instrument()]
    async fn setup(&mut self, config: &crate::config::Database) -> Result<()> {
        if cfg!(test) || config.clear_database_on_start {
            // TODO: do not do this if `clear_database_on_start` was false before restarting the server.

            self.drop_and_create_schema().await?;
        }

        if cfg!(test) {
            self.as_mut()
                .push_schema()
                .await
                .context("Failed to push schema to database")?;
        } else {
            self.apply_migrations().await?;
        }

        Ok(())
    }

    async fn drop_and_create_schema(&mut self) -> Result<()> {
        info!("Clearing database schema '{}'", self.schema);

        toasty::sql::statement(format!(
            "DROP SCHEMA IF EXISTS {schema} CASCADE;",
            schema = self.schema
        ))
        .exec(self.as_mut())
        .await
        .context("Failed to clear database schema")?;

        info!("Creating database schema '{}'", self.schema);

        toasty::sql::statement(format!(
            "CREATE SCHEMA IF NOT EXISTS {schema};",
            schema = self.schema
        ))
        .exec(self.as_mut())
        .await
        .context("Failed to create database schema")?;

        Ok(())
    }

    async fn apply_migrations(&mut self) -> Result<()> {
        let report = Self::MIGRATIONS
            .apply(self.as_mut())
            .await
            .context("Failed to apply migrations")?;

        info!("Applied {} database migrations", report.applied());
        Ok(())
    }

    /// Returns the name of the schema used by this database handle.
    #[cfg(test)]
    #[must_use]
    pub fn schema_name(&self) -> &str {
        &self.schema
    }

    #[must_use]
    pub fn db(&self) -> toasty::Db {
        self.db.clone()
    }
}
