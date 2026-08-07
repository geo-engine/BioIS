use chrono::{DateTime, Utc};
use o2o::o2o;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job status code enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed)]
#[serde(rename_all = "PascalCase")]
pub enum StatusCode {
    Accepted,
    Running,
    Successful,
    Failed,
    Dismissed,
}

impl From<StatusCode> for ogcapi::types::processes::StatusCode {
    fn from(val: StatusCode) -> Self {
        match val {
            StatusCode::Accepted => ogcapi::types::processes::StatusCode::Accepted,
            StatusCode::Running => ogcapi::types::processes::StatusCode::Running,
            StatusCode::Successful => ogcapi::types::processes::StatusCode::Successful,
            StatusCode::Failed => ogcapi::types::processes::StatusCode::Failed,
            StatusCode::Dismissed => ogcapi::types::processes::StatusCode::Dismissed,
        }
    }
}

impl From<ogcapi::types::processes::StatusCode> for StatusCode {
    fn from(val: ogcapi::types::processes::StatusCode) -> Self {
        match val {
            ogcapi::types::processes::StatusCode::Accepted => StatusCode::Accepted,
            ogcapi::types::processes::StatusCode::Running => StatusCode::Running,
            ogcapi::types::processes::StatusCode::Successful => StatusCode::Successful,
            ogcapi::types::processes::StatusCode::Failed => StatusCode::Failed,
            ogcapi::types::processes::StatusCode::Dismissed => StatusCode::Dismissed,
        }
    }
}

/// Job type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed)]
#[serde(rename_all = "PascalCase")]
pub enum JobType {
    Process,
}

impl From<JobType> for ogcapi::types::processes::JobType {
    fn from(val: JobType) -> Self {
        match val {
            JobType::Process => ogcapi::types::processes::JobType::Process,
        }
    }
}

impl From<ogcapi::types::processes::JobType> for JobType {
    fn from(val: ogcapi::types::processes::JobType) -> Self {
        match val {
            ogcapi::types::processes::JobType::Process => JobType::Process,
        }
    }
}

/// Response type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, toasty::Embed)]
#[serde(rename_all = "PascalCase")]
pub enum Response {
    Raw,
    Document,
}

impl From<Response> for ogcapi::types::processes::Response {
    fn from(val: Response) -> Self {
        match val {
            Response::Raw => ogcapi::types::processes::Response::Raw,
            Response::Document => ogcapi::types::processes::Response::Document,
        }
    }
}

impl From<ogcapi::types::processes::Response> for Response {
    fn from(val: ogcapi::types::processes::Response) -> Self {
        match val {
            ogcapi::types::processes::Response::Raw => Response::Raw,
            ogcapi::types::processes::Response::Document => Response::Document,
        }
    }
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
    pub job_id: uuid::Uuid,

    /// Referenced process ID
    pub process_id: Option<String>,

    /// Job type
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
    /// Convert created timestamp to DateTime<Utc>
    pub fn created_at(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_millis(self.created).unwrap_or_else(|| Utc::now())
    }

    /// Convert finished timestamp to DateTime<Utc>
    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        self.finished
            .and_then(|ts| DateTime::<Utc>::from_timestamp_millis(ts))
    }

    /// Convert updated timestamp to DateTime<Utc>
    pub fn updated_at(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_millis(self.updated).unwrap_or_else(|| Utc::now())
    }

    /// Set created from DateTime<Utc>
    pub fn with_created(mut self, dt: DateTime<Utc>) -> Self {
        self.created = dt.timestamp_millis();
        self
    }

    /// Set finished from DateTime<Utc>
    pub fn with_finished(mut self, dt: Option<DateTime<Utc>>) -> Self {
        self.finished = dt.map(|d| d.timestamp_millis());
        self
    }

    /// Set updated from DateTime<Utc>
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
