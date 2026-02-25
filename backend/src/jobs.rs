use crate::{
    db::{
        DbPool, PooledConnection,
        model::{self, DismissJob, NewJob, UpdateJob, UpdateJobStatus},
        schema::jobs,
    },
    state::USER,
};
use anyhow::Context;
use chrono::Utc;
use diesel::{ExpressionMethods, HasQuery, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ogcapi::{
    drivers::ProcessResult,
    types::{
        common::Link,
        processes::{ExecuteResults, Response, StatusCode, StatusInfo},
    },
};

pub struct JobHandler {
    connection: DbPool,
}

impl JobHandler {
    pub async fn new(connection: DbPool) -> anyhow::Result<Self> {
        let this = Self { connection };
        this.clean_running_jobs_from_previous_sessions().await?;
        Ok(this)
    }

    async fn connection(&self) -> anyhow::Result<PooledConnection<'_>> {
        self.connection
            .get()
            .await
            .context("could not get db connection from pool")
    }

    /// Clean up jobs that were in `Running` state from previous server sessions.
    /// Set them to `Failed` with appropriate message.
    async fn clean_running_jobs_from_previous_sessions(&self) -> anyhow::Result<()> {
        let update = UpdateJobStatus {
            status: model::StatusCode::Failed,
            message: "Server restarted during job execution".into(),
            updated: Utc::now(),
        };

        diesel::update(jobs::table)
            .filter(jobs::status.eq(model::StatusCode::Running))
            .set(update)
            .execute(&mut self.connection().await?)
            .await
            .context("Failed to update job in database")?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ogcapi::drivers::JobHandler for JobHandler {
    async fn register(&self, job: &StatusInfo, response_mode: Response) -> anyhow::Result<String> {
        let user = USER.try_get().context("missing authenticated user")?;

        let job_id = if job.job_id.is_empty() {
            uuid::Uuid::now_v7().to_string()
        } else {
            job.job_id.clone()
        };

        let new_job = NewJob {
            job_id: &job_id,
            process_id: job.process_id.as_deref(),
            status: job.status.clone().into(),
            message: job.message.as_deref(),
            job_type: job.r#type.clone().into(),
            created: job.created.unwrap_or_else(Utc::now),
            updated: job.updated.unwrap_or_else(Utc::now),
            progress: job.progress.map(Into::into),
            links: job.links.iter().map(|l| l.clone().into()).collect(),
            response: response_mode.into(),
            user_id: user.id,
        };

        let job_id: String = diesel::insert_into(jobs::table)
            .values(new_job)
            .returning(jobs::job_id)
            .get_result(&mut self.connection().await?)
            .await
            .context("Failed to insert job into database")?;

        Ok(job_id)
    }

    async fn update(&self, job: &StatusInfo) -> anyhow::Result<()> {
        let update = UpdateJob {
            status: job.status.clone().into(),
            message: job.message.as_deref(),
            updated: job.updated.unwrap_or_else(Utc::now),
            progress: job.progress.map(Into::into),
            links: job.links.iter().map(|l| l.clone().into()).collect(),
        };

        diesel::update(jobs::table)
            .filter(
                jobs::job_id.eq(&job.job_id), // .filter(jobs::user_id.eq(user.id)
            )
            .set(update)
            .execute(&mut self.connection().await?)
            .await
            .context("Failed to update job in database")?;

        Ok(())
    }

    async fn status_list(&self, offset: usize, limit: usize) -> anyhow::Result<Vec<StatusInfo>> {
        let user = USER.try_get().context("missing authenticated user")?;

        let query = model::StatusInfo::query()
            .filter(jobs::user_id.eq(user.id))
            .offset(offset as i64)
            .limit(limit as i64);

        let result = query
            .load::<model::StatusInfo>(&mut self.connection().await?)
            .await
            .context("Failed to query job status list from database")?;

        Ok(result
            .into_iter()
            .map(Into::into)
            .collect::<Vec<StatusInfo>>())
    }

    async fn status(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let user = USER.try_get().context("missing authenticated user")?;

        model::StatusInfo::query()
            .filter(jobs::job_id.eq(id))
            .filter(jobs::user_id.eq(user.id))
            .first(&mut self.connection().await?)
            .await
            .optional()
            .map(|s| s.map(Into::into))
            .context("Failed to query job status from database")
    }

    async fn finish(
        &self,
        job_id: &str,
        status: &StatusCode,
        message: Option<String>,
        links: Vec<Link>,
        results: Option<ExecuteResults>,
    ) -> anyhow::Result<()> {
        let finish = crate::db::model::FinishJob {
            status: status.clone().into(),
            message: message.as_deref(),
            updated: Utc::now(),
            finished: Utc::now(),
            progress: Some(100),
            links: links.iter().map(|l| l.clone().into()).collect(),
            results: results.map(serde_json::to_value).transpose()?,
        };

        diesel::update(
            jobs::table.filter(jobs::job_id.eq(job_id)), // .filter(jobs::user_id.eq(user.id)),
        )
        .set(finish)
        .execute(&mut self.connection().await?)
        .await
        .context("Failed write finished job to database")?;

        Ok(())
    }

    async fn dismiss(&self, id: &str) -> anyhow::Result<Option<StatusInfo>> {
        let user = USER.try_get().context("missing authenticated user")?;

        let returned: Option<model::StatusInfo> = diesel::update(jobs::table)
            .filter(jobs::job_id.eq(id))
            .filter(jobs::user_id.eq(user.id))
            .set(DismissJob {
                status: StatusCode::Dismissed.into(),
                message: Some("Job dismissed by user"),
                updated: Utc::now(),
            })
            .returning(model::StatusInfo::as_returning())
            .get_result(&mut self.connection().await?)
            .await
            .optional()
            .context("Failed to dismiss job in database")?;

        Ok(returned.map(Into::into))
    }

    async fn results(&self, id: &str) -> anyhow::Result<ProcessResult> {
        let user = USER.try_get().context("missing authenticated user")?;

        let results: Option<(Option<serde_json::Value>, model::Response)> = jobs::table
            .select((jobs::results, jobs::response))
            .filter(jobs::job_id.eq(id))
            .filter(jobs::user_id.eq(user.id))
            .first(&mut self.connection().await?)
            .await
            .optional()
            .context("Failed to query job results from database")?;

        let Some((results, response_mode)) = results else {
            return Ok(ProcessResult::NoSuchJob);
        };

        let Some(results) = results else {
            return Ok(ProcessResult::NotReady);
        };

        let results: ExecuteResults = serde_json::from_value(results)?;

        Ok(ProcessResult::Results {
            results,
            response_mode: response_mode.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{auth::User, config::CONFIG, db::setup_db};
    use ogcapi::drivers::JobHandler as _;

    async fn mock_db_pool() -> DbPool {
        setup_db(&CONFIG.database).await.unwrap()
    }

    fn mock_user() -> User {
        User {
            id: uuid::Uuid::from_u128(0xabcd_efab_cdef_abcd_efab_cdef_abcd_efab),
            session_token: uuid::Uuid::from_u128(0x1234_5678_90ab_cdef_1234_5678_90ab_cdef).into(),
        }
    }

    fn mock_status_info(job_id: &str) -> StatusInfo {
        StatusInfo {
            job_id: job_id.to_string(),
            process_id: Some("proc".to_string()),
            status: StatusCode::Accepted,
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
    async fn test_register() {
        let pool = mock_db_pool().await;
        let handler = JobHandler::new(pool).await.unwrap();
        USER.scope(mock_user(), async move {
            let status_info = mock_status_info("");
            let result = handler.register(&status_info, Response::Raw).await;
            assert!(result.is_ok());
            let job_id = result.unwrap();
            assert!(!job_id.is_empty());
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_update() {
        let pool = mock_db_pool().await;
        let handler = JobHandler::new(pool).await.unwrap();
        USER.scope(mock_user(), async move {
            let status_info = mock_status_info("job1");
            // Register first
            handler.register(&status_info, Response::Raw).await.unwrap();
            // Update
            let mut updated_info = status_info.clone();
            updated_info.status = StatusCode::Running;
            let result = handler.update(&updated_info).await;
            assert!(result.is_ok());
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_status_list() {
        let pool = mock_db_pool().await;
        let handler = JobHandler::new(pool).await.unwrap();
        USER.scope(mock_user(), async move {
            // Register a job
            let status_info = mock_status_info("job2");
            handler.register(&status_info, Response::Raw).await.unwrap();
            // List
            let result = handler.status_list(0, 10).await;
            assert!(result.is_ok());
            let list = result.unwrap();
            assert!(!list.is_empty());
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_status() {
        let pool = mock_db_pool().await;
        let handler = JobHandler::new(pool).await.unwrap();
        USER.scope(mock_user(), async move {
            let status_info = mock_status_info("job3");
            handler.register(&status_info, Response::Raw).await.unwrap();
            let result = handler.status("job3").await;
            assert!(result.is_ok());
            let status = result.unwrap();
            assert!(status.is_some());
            assert_eq!(status.unwrap().job_id, "job3");
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_finish() {
        let pool = mock_db_pool().await;
        let handler = JobHandler::new(pool).await.unwrap();
        USER.scope(mock_user(), async move {
            let status_info = mock_status_info("job4");
            handler.register(&status_info, Response::Raw).await.unwrap();
            let result = handler
                .finish(
                    "job4",
                    &StatusCode::Successful,
                    Some("done".to_string()),
                    vec![],
                    None,
                )
                .await;
            assert!(result.is_ok());
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_dismiss() {
        let pool = mock_db_pool().await;
        let handler = JobHandler::new(pool).await.unwrap();
        USER.scope(mock_user(), async move {
            let status_info = mock_status_info("job5");
            handler.register(&status_info, Response::Raw).await.unwrap();
            let result = handler.dismiss("job5").await;
            assert!(result.is_ok());
            let dismissed = result.unwrap();
            assert!(dismissed.is_some());
            assert_eq!(dismissed.unwrap().status, StatusCode::Dismissed);
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_results_no_job() {
        let pool = mock_db_pool().await;
        let handler = JobHandler::new(pool).await.unwrap();
        USER.scope(mock_user(), async move {
            let result = handler.results("no_such_job").await;
            assert!(matches!(result.unwrap(), ProcessResult::NoSuchJob));
        })
        .await;
    }
}
