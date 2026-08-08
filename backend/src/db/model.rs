use chrono::{DateTime, Utc};
use o2o::o2o;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job status code enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed, o2o)]
#[serde(rename_all = "PascalCase")]
#[from_owned(ogcapi::types::processes::StatusCode)]
#[owned_into(ogcapi::types::processes::StatusCode)]
#[column(type = text)]
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
pub enum JobType {
    Process,
}

/// Response type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed, o2o)]
#[serde(rename_all = "PascalCase")]
#[from_owned(ogcapi::types::processes::Response)]
#[owned_into(ogcapi::types::processes::Response)]
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
    pub created: i64,

    /// Job completion timestamp (stored as Unix timestamp in milliseconds)
    pub finished: Option<i64>,

    /// Last update timestamp (stored as Unix timestamp in milliseconds)
    pub updated: i64,

    /// Progress percentage (0-100)
    pub progress: Option<i16>,

    /// Links associated with the job (stored as JSONB)
    pub links: Vec<Link>,

    /// Response type
    pub response: Response,

    /// Job results (JSONB)
    #[column(type = "jsonb")]
    pub results: Option<serde_json::Value>,

    /// User ID who created the job
    pub user_id: uuid::Uuid,

    #[has_many]
    pub credits: toasty::Deferred<Vec<Credits>>,
}

impl Job {
    /// Convert created timestamp to `DateTime<Utc>`
    pub fn created_at(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_millis(self.created).unwrap_or_else(Utc::now)
    }

    /// Convert finished timestamp to `DateTime<Utc>`
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        self.finished
            .and_then(DateTime::<Utc>::from_timestamp_millis)
    }

    /// Convert updated timestamp to `DateTime<Utc>`
    pub fn updated_at(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_millis(self.updated).unwrap_or_else(Utc::now)
    }

    /// Set created from `DateTime<Utc>`
    pub fn with_created(mut self, dt: DateTime<Utc>) -> Self {
        self.created = dt.timestamp_millis();
        self
    }

    /// Set finished from `DateTime<Utc>`
    pub fn with_finished(mut self, dt: Option<DateTime<Utc>>) -> Self {
        self.finished = dt.map(|d| d.timestamp_millis());
        self
    }

    /// Set updated from `DateTime<Utc>`
    pub fn with_updated(mut self, dt: DateTime<Utc>) -> Self {
        self.updated = dt.timestamp_millis();
        self
    }
}

/// Credits database model
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[key(job_id, compute_id)] // Note: `toasty` requires a primary key, but we would omit this normally.
pub struct Credits {
    pub timestamp: i64,

    /// Referenced job ID
    #[index]
    pub job_id: uuid::Uuid,

    /// Referenced job
    #[belongs_to(key = job_id, references = job_id)]
    pub job: toasty::Deferred<Job>,

    /// Geo Engine compute ID (if applicable)
    ///
    /// Note: Not stored as nullable because `toasty` requires a primary key.
    pub compute_id: ComputeId,

    /// Credits used; empty if not yet determined (e.g., job still running)
    #[allow(clippy::struct_field_names, reason = "Database field name")]
    pub credits: Option<u64>,
}

/// An optional compute ID for Geo Engine jobs, stored as a string.
/// This is used to track the compute resources used by a job in the Geo Engine system.
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Embed)]
pub struct ComputeId(Uuid);

impl ComputeId {
    pub fn some(id: Uuid) -> Self {
        Self(id)
    }

    pub fn none() -> Self {
        Self(Uuid::nil())
    }

    pub fn get(&self) -> Option<Uuid> {
        if self.0.is_nil() {
            return None;
        }
        Some(self.0)
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
