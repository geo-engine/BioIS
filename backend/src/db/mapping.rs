//! Mapping between database models and OGC API types

use crate::db::model::{JobType, Link, Response, StatusCode, StatusInfo};
use ogcapi::types::{
    common::Link as OgcApiLink,
    processes::{
        JobType as OgcApiJobType, Response as OgcApiResponse, StatusCode as OgcApiStatusCode,
        StatusInfo as OgcApiStatusInfo,
    },
};

impl From<Response> for OgcApiResponse {
    fn from(response: Response) -> Self {
        match response {
            Response::Raw => OgcApiResponse::Raw,
            Response::Document => OgcApiResponse::Document,
        }
    }
}

impl From<OgcApiResponse> for Response {
    fn from(response: OgcApiResponse) -> Self {
        match response {
            OgcApiResponse::Raw => Self::Raw,
            OgcApiResponse::Document => Self::Document,
        }
    }
}

impl From<OgcApiLink> for Link {
    fn from(link: OgcApiLink) -> Self {
        Link {
            href: link.href,
            rel: link.rel,
            r#type: link.r#type,
            hreflang: link.hreflang,
            title: link.title,
            length: link.length,
        }
    }
}

impl From<Link> for OgcApiLink {
    fn from(link: Link) -> Self {
        OgcApiLink {
            href: link.href,
            rel: link.rel,
            r#type: link.r#type,
            hreflang: link.hreflang,
            title: link.title,
            length: link.length,
        }
    }
}

impl From<JobType> for OgcApiJobType {
    fn from(job_type: JobType) -> Self {
        match job_type {
            JobType::Process => Self::Process,
        }
    }
}

impl From<OgcApiJobType> for JobType {
    fn from(job_type: OgcApiJobType) -> Self {
        match job_type {
            OgcApiJobType::Process => Self::Process,
        }
    }
}

impl From<StatusCode> for OgcApiStatusCode {
    fn from(status: StatusCode) -> Self {
        match status {
            StatusCode::Accepted => OgcApiStatusCode::Accepted,
            StatusCode::Running => OgcApiStatusCode::Running,
            StatusCode::Successful => OgcApiStatusCode::Successful,
            StatusCode::Failed => OgcApiStatusCode::Failed,
            StatusCode::Dismissed => OgcApiStatusCode::Dismissed,
        }
    }
}

impl From<OgcApiStatusCode> for StatusCode {
    fn from(status: OgcApiStatusCode) -> Self {
        match status {
            OgcApiStatusCode::Accepted => StatusCode::Accepted,
            OgcApiStatusCode::Running => StatusCode::Running,
            OgcApiStatusCode::Successful => StatusCode::Successful,
            OgcApiStatusCode::Failed => StatusCode::Failed,
            OgcApiStatusCode::Dismissed => StatusCode::Dismissed,
        }
    }
}

impl From<StatusInfo> for OgcApiStatusInfo {
    fn from(status: StatusInfo) -> Self {
        OgcApiStatusInfo {
            process_id: status.process_id,
            r#type: status.job_type.into(),
            job_id: status.job_id,
            status: status.status.into(),
            message: status.message,
            created: Some(status.created),
            finished: status.finished,
            updated: Some(status.updated),
            progress: status.progress.map(|p| p as u8),
            links: status
                .links
                .into_iter()
                .filter_map(|l| l.map(Into::into))
                .collect(),
        }
    }
}
