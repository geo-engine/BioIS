#![allow(
    clippy::used_underscore_binding,
    reason = "For embedded NewTypes, toasty accesses the inner value via `_0`, cf. <https://github.com/tokio-rs/toasty/issues/1179>"
)]

use anyhow::Context;
use chrono::{DateTime, Utc};
use o2o::o2o;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// Job status code enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed, o2o)]
#[serde(rename_all = "PascalCase")]
#[from_owned(ogcapi::types::processes::StatusCode)]
#[owned_into(ogcapi::types::processes::StatusCode)]
// #[column(type = enum("StatusCode"))]
pub enum StatusCode {
    Accepted,
    Running,
    Successful,
    Failed,
    Dismissed,
}

/// Job type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed, o2o)]
#[serde(rename_all = "PascalCase")]
#[from_owned(ogcapi::types::processes::JobType)]
#[owned_into(ogcapi::types::processes::JobType)]
// #[column(type = enum("JobType"))]
pub enum JobType {
    Process,
}

/// Response type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed, o2o)]
#[serde(rename_all = "PascalCase")]
#[from_owned(ogcapi::types::processes::Response)]
#[owned_into(ogcapi::types::processes::Response)]
// #[column(type = enum("Response"))]
pub enum Response {
    Raw,
    Document,
}

/// Link reference (stored as JSONB)
#[derive(Debug, Clone, Serialize, Deserialize, o2o, toasty::Embed)]
#[from_owned(ogcapi::types::common::Link)]
#[owned_into(ogcapi::types::common::Link)]
#[ghosts(templated: None, var_base: None)]
pub struct Link {
    pub href: String,
    pub rel: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<i64>,
}

/// Job database model
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
pub struct Job {
    /// Primary key: job ID
    #[key]
    #[auto(uuid(v7))]
    #[allow(clippy::struct_field_names, reason = "Database field name")]
    pub job_id: uuid::Uuid,

    /// Referenced process ID
    pub process_id: Option<String>,

    /// Job type
    #[allow(clippy::struct_field_names, reason = "Database field name")]
    pub job_type: JobType,

    /// Current status
    pub status: StatusCode,

    /// Status message
    pub message: Option<String>,

    /// Job creation timestamp (stored as Unix timestamp in milliseconds)
    #[default(TimestampMillis::now())]
    pub created: TimestampMillis,

    /// Job completion timestamp (stored as Unix timestamp in milliseconds)
    #[default(None)]
    pub finished: Option<TimestampMillis>,

    /// Last update timestamp (stored as Unix timestamp in milliseconds)
    #[update(TimestampMillis::now())]
    pub updated: TimestampMillis,

    /// Progress percentage (0-100)
    pub progress: Option<i16>,

    /// Links associated with the job
    pub links: Vec<Link>,

    /// Response type
    pub response: Response,

    /// Job results
    /// TODO: make enum of all possible result types
    #[column(type = "jsonb")]
    pub results: Option<serde_json::Value>,

    /// User ID who created the job
    pub user_id: uuid::Uuid,

    #[has_many]
    pub credits: toasty::Deferred<Vec<Credits>>,
}

/// A timestamp stored as a Unix timestamp in milliseconds, used for database storage.
///
/// TODO: Remove once we can use `jiff` or `DateTime` in `toasty` models
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, toasty::Embed,
)]
pub struct TimestampMillis(i64);

impl TimestampMillis {
    #[must_use]
    pub fn now() -> Self {
        Self(Utc::now().timestamp_millis())
    }
}

impl From<DateTime<Utc>> for TimestampMillis {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt.timestamp_millis())
    }
}

impl TryInto<DateTime<Utc>> for TimestampMillis {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<DateTime<Utc>, Self::Error> {
        DateTime::<Utc>::from_timestamp_millis(self.0).context("invalid timestamp")
    }
}

/// Credits database model
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[key(job_id, computation_id)] // Note: `toasty` requires a primary key, but we would omit this normally.
pub struct Credits {
    pub timestamp: TimestampMillis,

    /// Referenced job ID
    #[index]
    pub job_id: uuid::Uuid,

    /// Referenced job
    #[belongs_to(key = job_id, references = job_id)]
    pub job: toasty::Deferred<Job>,

    /// Geo Engine computation ID (if applicable)
    ///
    /// Note: Not stored as nullable because `toasty` requires a primary key.
    pub computation_id: ComputationId,

    /// Credits used; empty if not yet determined (e.g., job still running)
    #[allow(clippy::struct_field_names, reason = "Database field name")]
    pub credits: Option<u64>,
}

/// An optional computation ID for Geo Engine jobs, stored as a string.
/// This is used to track the compute resources used by a job in the Geo Engine system.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, toasty::Embed)]
pub struct ComputationId(Uuid);

impl ComputationId {
    #[must_use]
    pub fn some(id: Uuid) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn none() -> Self {
        Self(Uuid::nil())
    }

    #[must_use]
    pub fn get(&self) -> Option<Uuid> {
        if self.is_none() {
            return None;
        }
        Some(self.0)
    }

    #[must_use]
    pub fn is_none(&self) -> bool {
        self.0.is_nil()
    }

    #[must_use]
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

impl FromStr for ComputationId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::parse_str(s).context("invalid UUID string")?;
        Ok(Self(id))
    }
}

impl std::fmt::Display for ComputationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(id) = self.get() {
            write!(f, "{id}")
        } else {
            write!(f, "None")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_jobtype_from_string() {
        let v: JobType = serde_json::from_str("\"Process\"").expect("to deserialize JobType");
        assert!(matches!(v, JobType::Process));
    }

    #[test]
    fn deserialize_statuscode_variants() {
        let s = serde_json::from_str::<StatusCode>("\"Accepted\"").expect("accepted");
        assert!(matches!(s, StatusCode::Accepted));

        let s = serde_json::from_str::<StatusCode>("\"Running\"").expect("running");
        assert!(matches!(s, StatusCode::Running));

        let s = serde_json::from_str::<StatusCode>("\"Successful\"").expect("successful");
        assert!(matches!(s, StatusCode::Successful));

        let s = serde_json::from_str::<StatusCode>("\"Failed\"").expect("failed");
        assert!(matches!(s, StatusCode::Failed));

        let s = serde_json::from_str::<StatusCode>("\"Dismissed\"").expect("dismissed");
        assert!(matches!(s, StatusCode::Dismissed));
    }

    #[test]
    fn deserialize_response_enum() {
        let r: Response = serde_json::from_str("\"Raw\"").expect("raw");
        assert!(matches!(r, Response::Raw));
    }
}
