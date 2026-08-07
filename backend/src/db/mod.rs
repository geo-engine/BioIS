use anyhow::{Context, Result};
use std::{
    ops::{Deref, DerefMut},
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
    schema: String,
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
    /// Returns the name of the schema used by this database handle.
    pub fn schema_name(&self) -> &str {
        &self.schema
    }

    pub fn db(&self) -> toasty::Db {
        self.db.clone()
    }
}

/// Create a database connection and initialize the schema
#[instrument()]
pub async fn setup_db(config: &crate::config::Database) -> Result<DbHandle> {
    let mut db = toasty::Db::builder()
        .models(toasty::models!(model::Job))
        // .table_name_prefix(&format!("{}.", config.schema))
        .max_pool_size(if cfg!(test) { 1 } else { 8 })
        .pool_wait_timeout(Some(Duration::from_secs(5)))
        .pool_create_timeout(Some(Duration::from_secs(10)))
        .pool_health_check_interval(Some(Duration::from_secs(60)))
        .connect(&config.connection_string())
        .await
        .context("Failed to connect to database")?;

    on_startup(&mut db, config).await?;

    Ok(DbHandle {
        db,
        schema: config.schema.clone(),
    })
}

async fn on_startup(db: &mut toasty::Db, config: &crate::config::Database) -> Result<()> {
    // For development, use push_schema to quickly set up tables
    // For production, migrations should be applied separately via CLI

    if cfg!(test) || config.clear_database_on_start {
        if config.clear_database_on_start {
            info!("Clearing database schema '{}'", config.schema);
            // Drop and recreate schema
            toasty::sql::statement(format!("DROP SCHEMA IF EXISTS {} CASCADE;", &config.schema))
                .exec(db)
                .await
                .context("Failed to clear database schema")?;
        }

        info!("Creating database schema '{}'", config.schema);
        toasty::sql::statement(format!("CREATE SCHEMA IF NOT EXISTS {};", &config.schema))
            .exec(db)
            .await
            .context("Failed to create database schema")?;

        toasty::sql::statement(&format!(
            r#"SET search_path TO "{schema_name}", public;"#,
            schema_name = &config.schema
        ))
        .exec(db)
        .await
        .context("Failed to set search_path for database schema")?;

        if cfg!(test) {
            let mut connection = db.connection().await?;

            toasty::sql::statement(&format!(
                r#"SET search_path TO "{schema_name}", public;"#,
                schema_name = &config.schema
            ))
            .exec(&mut connection)
            .await
            .context("Failed to set search_path for database schema")?;

            connection
                .push_schema()
                .await
                .context("Failed to push schema to database")?;
        }
    }

    // TODO: In production, use proper migrations

    Ok(())
}
