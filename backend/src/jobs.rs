use crate::{
    db::{
        DbHandle,
        model::{Job, JobType, Link, Response, StatusCode, TimestampMillis, job_updated_field},
    },
    state::{CONTEXT, TaskLocalContext},
};
use anyhow::Context;
use chrono::Utc;
use ogcapi::{
    drivers::ProcessResult,
    types::{
        common::Link as ApiLink,
        processes::{
            ExecuteResults, Response as ApiResponse, StatusCode as ApiStatusCode, StatusInfo,
        },
    },
};
use tracing::instrument;
use uuid::Uuid;

pub struct JobHandler {
    db: DbHandle,
}

impl JobHandler {
    pub async fn new(db: DbHandle) -> anyhow::Result<Self> {
        let mut this = Self { db };
        this.clean_running_jobs_from_previous_sessions().await?;
        Ok(this)
    }

    /// Clean up jobs that were in `Running` state from previous server sessions.
    /// Set them to `Failed` with appropriate message.
    #[instrument(skip(self), level = "debug")]
    async fn clean_running_jobs_from_previous_sessions(&mut self) -> anyhow::Result<()> {
        let now_ms: TimestampMillis = Utc::now().into();

        toasty::update!(Job::filter(
            Job::fields().status().eq(StatusCode::Running)
        ) {
            status: StatusCode::Failed,
            message: Some("Server restarted during job execution".to_string()),
            updated: now_ms,
        })
        .exec(&mut self.db.db())
        .await
        .context("Failed to clean up running jobs")
    }
}

#[async_trait::async_trait]
impl ogcapi::drivers::JobHandler for JobHandler {
    async fn register(
        &self,
        job: &StatusInfo,
        response_mode: ApiResponse,
    ) -> anyhow::Result<String> {
        let user_id = CONTEXT.user_id()?;

        if !job.job_id.is_empty() {
            anyhow::bail!("Job ID must be empty when registering a new job");
        }

        let now_ms: TimestampMillis = Utc::now().into();
        let created_ms = job.created.map_or(now_ms, Into::into);
        let updated_ms = job.updated.map_or(now_ms, Into::into);

        let job = toasty::create!(Job {
            process_id: job.process_id.clone(),
            job_type: JobType::from(job.r#type.clone()),
            status: StatusCode::from(job.status.clone()),
            message: job.message.clone(),
            created: created_ms,
            finished: None,
            updated: updated_ms,
            progress: job.progress.map(Into::into),
            links: job
                .links
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<Link>>(),
            response: Response::from(response_mode),
            results: None,
            user_id,
        })
        .exec(&mut self.db.db())
        .await
        .context("Failed to insert job into database")?;

        CONTEXT.set_job_id(job.job_id)?;

        Ok(job.job_id.to_string())
    }

    #[instrument(skip(self, job), level = "debug")]
    async fn update(&self, job: &StatusInfo) -> anyhow::Result<()> {
        let job_id = Uuid::parse_str(&job.job_id).context("Invalid job ID format")?;
        let now_ms: TimestampMillis = Utc::now().into();
        let updated_ms = job.updated.map_or(now_ms, Into::into);
        let progress = job.progress.map_or(0, i16::from);

        toasty::update!(Job::filter_by_job_id(job_id) {
            status: StatusCode::from(job.status.clone()),
            message: job.message.clone(),
            updated: updated_ms,
            progress: Some(progress),
        })
        .exec(&mut self.db.db())
        .await
        .context("Failed to update job in database")?;

        CONTEXT.set_job_id(job_id)?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn status_list(&self, offset: usize, limit: usize) -> anyhow::Result<Vec<StatusInfo>> {
        let user_id = CONTEXT.user_id()?;

        let results: Vec<Job> = Job::filter(Job::fields().user_id().eq(user_id))
            .order_by(job_updated_field().desc())
            .limit(limit)
            .offset(offset)
            .exec(&mut self.db.db())
            .await
            .context("Failed to query job status list from database")?;

        Ok(results
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<StatusInfo>, _>>()?)
    }

    #[instrument(skip(self), level = "debug")]
    async fn status(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let user_id = CONTEXT.user_id()?;
        let id = Uuid::parse_str(id).context("Invalid job ID format")?;

        let result: Option<Job> = Job::filter_by_job_id(id)
            .filter(Job::fields().user_id().eq(user_id))
            .first()
            .exec(&mut self.db.db())
            .await
            .context("Failed to query job status from database")?;

        Ok(result.map(TryInto::try_into).transpose()?)
    }

    #[instrument(skip(self, message, links, results), level = "debug")]
    async fn finish(
        &self,
        job_id: &str,
        status: &ApiStatusCode,
        message: Option<String>,
        links: Vec<ApiLink>,
        results: Option<ExecuteResults>,
    ) -> anyhow::Result<()> {
        let now_ms: TimestampMillis = Utc::now().into();
        let job_id = Uuid::parse_str(job_id).context("Invalid job ID format")?;
        let results_json = results.map(serde_json::to_value).transpose()?;

        toasty::update!(Job::filter_by_job_id(job_id) {
            status: StatusCode::from(status.clone()),
            message,
            updated: now_ms,
            finished: Some(now_ms),
            progress: Some(100i16),
            links: links.iter().cloned().map(Into::into).collect::<Vec<Link>>(),
            results: results_json,
        })
        .exec(&mut self.db.db())
        .await
        .context("Failed to finish job in database")?;

        CONTEXT.set_job_id(job_id)?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn dismiss(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let user_id = CONTEXT.user_id()?;
        let now_ms: TimestampMillis = Utc::now().into();
        let id = Uuid::parse_str(id).context("Invalid job ID format")?;

        // First, get the current job to return
        let current_job: Option<Job> = Job::filter_by_job_id(id)
            .filter(Job::fields().user_id().eq(user_id))
            .first()
            .exec(&mut self.db.db())
            .await
            .context("Failed to query current job before dismiss")?;

        // Update the job to dismissed status
        toasty::update!(Job::filter(
            Job::fields().job_id().eq(id)
        )
        .filter(Job::fields().user_id().eq(user_id)) {
            status: StatusCode::Dismissed,
            message: Some("Job dismissed by user".to_string()),
            updated: now_ms,
        })
        .exec(&mut self.db.db())
        .await
        .context("Failed to dismiss job in database")?;

        // Return the updated job info
        let Some(mut job) = current_job else {
            return Ok(None);
        };
        job.status = StatusCode::Dismissed;
        job.message = Some("Job dismissed by user".to_string());
        job.updated = now_ms;
        job.try_into().map(Some)
    }

    #[instrument(skip(self), level = "debug")]
    async fn results(&self, id: &str) -> anyhow::Result<ProcessResult> {
        let user_id = CONTEXT.user_id()?;
        let id = Uuid::parse_str(id).context("Invalid job ID format")?;

        let result: Option<Job> = Job::filter_by_job_id(id)
            .filter(Job::fields().user_id().eq(user_id))
            .first()
            .exec(&mut self.db.db())
            .await
            .context("Failed to query job results from database")?;

        let Some(job) = result else {
            return Ok(ProcessResult::NoSuchJob);
        };

        let Some(results_json) = job.results else {
            return Ok(ProcessResult::NotReady);
        };

        let results: ExecuteResults = serde_json::from_value(results_json)?;
        let response_mode = job.response;

        Ok(ProcessResult::Results {
            results,
            response_mode: response_mode.into(),
        })
    }
}

impl TryInto<StatusInfo> for Job {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<StatusInfo, Self::Error> {
        Ok(StatusInfo {
            job_id: self.job_id.to_string(),
            process_id: self.process_id,
            status: self.status.into(),
            message: self.message,
            r#type: self.job_type.into(),
            created: Some(self.created.try_into()?),
            updated: Some(self.updated.try_into()?),
            finished: self.finished.map(TryInto::try_into).transpose()?,
            progress: self.progress.map(|p| p as u8),
            links: self.links.into_iter().map(Into::into).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{auth::User, db::tests::with_temp_db, state::TaskContext};
    use ogcapi::drivers::JobHandler as _;

    fn mock_user() -> User {
        User {
            id: uuid::Uuid::from_u128(0xabcd_efab_cdef_abcd_efab_cdef_abcd_efab),
            session_token: uuid::Uuid::from_u128(0x1234_5678_90ab_cdef_1234_5678_90ab_cdef).into(),
        }
    }

    fn mock_status_info() -> StatusInfo {
        StatusInfo {
            job_id: String::new(), // job_id must be empty when registering a new job
            process_id: Some("proc".to_string()),
            status: ApiStatusCode::Accepted,
            message: Some("msg".to_string()),
            r#type: Default::default(),
            created: Some(Utc::now()),
            updated: Some(Utc::now()),
            progress: Some(10),
            links: vec![],
            finished: None,
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn it_registers_jobs() {
        with_temp_db(|db_pool| async move {
            let handler = JobHandler::new(db_pool).await.unwrap();
            CONTEXT
                .scope(TaskContext::new(mock_user()), async move {
                    let status_info = mock_status_info();
                    let result = handler.register(&status_info, ApiResponse::Raw).await;
                    assert!(result.is_ok());
                    let job_id = result.unwrap();
                    assert!(!job_id.is_empty());
                })
                .await;
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_update() {
        with_temp_db(|db_pool| async move {
            let handler = JobHandler::new(db_pool).await.unwrap();
            CONTEXT
                .scope(TaskContext::new(mock_user()), async move {
                    let status_info = mock_status_info();
                    // Register first
                    let job_id = handler
                        .register(&status_info, ApiResponse::Raw)
                        .await
                        .unwrap();
                    // Update
                    let mut updated_info = status_info.clone();
                    updated_info.job_id = job_id;
                    updated_info.status = ApiStatusCode::Running;
                    let result = handler.update(&updated_info).await;
                    assert!(result.is_ok());
                })
                .await;
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_status_list() {
        with_temp_db(|db_pool| async move {
            let handler = JobHandler::new(db_pool).await.unwrap();
            CONTEXT
                .scope(TaskContext::new(mock_user()), async move {
                    // Register a job
                    let status_info = mock_status_info();
                    handler
                        .register(&status_info, ApiResponse::Raw)
                        .await
                        .unwrap();
                    // List
                    let result = handler.status_list(0, 10).await;
                    assert!(result.is_ok());
                    let list = result.unwrap();
                    assert!(!list.is_empty());
                })
                .await;
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_status() {
        with_temp_db(|db_pool| async move {
            let handler = JobHandler::new(db_pool).await.unwrap();
            CONTEXT
                .scope(TaskContext::new(mock_user()), async move {
                    let status_info = mock_status_info();
                    let job_id = handler
                        .register(&status_info, ApiResponse::Raw)
                        .await
                        .unwrap();
                    let result = handler.status(&job_id).await;
                    assert!(result.is_ok());
                    let status = result.unwrap();
                    assert!(status.is_some());
                    assert_eq!(status.unwrap().job_id, job_id);
                })
                .await;
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_finish() {
        with_temp_db(|db_pool| async move {
            let handler = JobHandler::new(db_pool).await.unwrap();
            CONTEXT
                .scope(TaskContext::new(mock_user()), async move {
                    let status_info = mock_status_info();
                    let job_id = handler
                        .register(&status_info, ApiResponse::Raw)
                        .await
                        .unwrap();
                    let result = handler
                        .finish(
                            &job_id,
                            &ApiStatusCode::Successful,
                            Some("done".to_string()),
                            vec![],
                            None,
                        )
                        .await;
                    assert!(result.is_ok());
                })
                .await;
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_dismiss() {
        with_temp_db(|db_pool| async move {
            let handler = JobHandler::new(db_pool).await.unwrap();
            CONTEXT
                .scope(TaskContext::new(mock_user()), async move {
                    let status_info = mock_status_info();
                    let job_id = handler
                        .register(&status_info, ApiResponse::Raw)
                        .await
                        .unwrap();
                    let result = handler.dismiss(&job_id).await;
                    assert!(result.is_ok());
                    let dismissed = result.unwrap();
                    assert!(dismissed.is_some());
                    assert_eq!(dismissed.unwrap().status, ApiStatusCode::Dismissed);
                })
                .await;
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_results_no_job() {
        with_temp_db(|db_pool| async move {
            let handler = JobHandler::new(db_pool).await.unwrap();
            CONTEXT
                .scope(TaskContext::new(mock_user()), async move {
                    let non_existent_id = "00000000-0000-0000-0000-000000000000";
                    let result = handler.results(non_existent_id).await;
                    assert!(matches!(result.unwrap(), ProcessResult::NoSuchJob));
                })
                .await;
        })
        .await;
    }
}
