use crate::{
    auth::User,
    db::{
        DbPool,
        model::{self, DismissJob, NewJob, UpdateJob},
        schema::jobs,
    },
};
use anyhow::Context;
use chrono::Utc;
use diesel::{
    ExpressionMethods, HasQuery, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use ogcapi::{
    drivers::ProcessResult,
    types::{
        common::Link,
        processes::{ExecuteResults, Response, StatusCode, StatusInfo},
    },
};

pub struct JobHandler {
    pub(crate) connection: DbPool,
}

#[async_trait::async_trait]
impl ogcapi::drivers::JobHandler for JobHandler {
    type User = User;

    async fn register(
        &self,
        job: &StatusInfo,
        response_mode: Response,
        user: &Self::User,
    ) -> anyhow::Result<String> {
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

        let mut connection = self
            .connection
            .get()
            .context("could not get db connection from pool")?;

        let job_id: String = diesel::insert_into(jobs::table)
            .values(new_job)
            .returning(jobs::job_id)
            .get_result(&mut connection)
            .context("Failed to insert job into database")?;

        Ok(job_id)
    }

    async fn update(&self, job: &StatusInfo, user: &Self::User) -> anyhow::Result<()> {
        let update = UpdateJob {
            status: job.status.clone().into(),
            message: job.message.as_deref(),
            updated: job.updated.unwrap_or_else(Utc::now),
            progress: job.progress.map(Into::into),
            links: job.links.iter().map(|l| l.clone().into()).collect(),
        };

        let mut connection = self
            .connection
            .get()
            .context("could not get db connection from pool")?;

        diesel::update(jobs::table)
            .filter(jobs::job_id.eq(&job.job_id))
            .filter(jobs::user_id.eq(user.id))
            .set(update)
            .execute(&mut connection)
            .context("Failed to update job in database")?;

        Ok(())
    }

    async fn status_list(
        &self,
        offset: usize,
        limit: usize,
        user: &Self::User,
    ) -> anyhow::Result<Vec<StatusInfo>> {
        let mut connection = self
            .connection
            .get()
            .context("could not get db connection from pool")?;

        let result = model::StatusInfo::query()
            .filter(jobs::user_id.eq(user.id))
            .offset(offset as i64)
            .limit(limit as i64)
            .load::<model::StatusInfo>(&mut connection)
            .context("Failed to query job status list from database")?;

        Ok(result
            .into_iter()
            .map(Into::into)
            .collect::<Vec<StatusInfo>>())
    }

    async fn status(&self, id: &str, user: &Self::User) -> anyhow::Result<Option<StatusInfo>> {
        let mut connection = self
            .connection
            .get()
            .context("could not get db connection from pool")?;

        model::StatusInfo::query()
            .filter(jobs::job_id.eq(id))
            .filter(jobs::user_id.eq(user.id))
            .first(&mut connection)
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
        user: &Self::User,
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

        let mut connection = self
            .connection
            .get()
            .context("could not get db connection from pool")?;

        diesel::update(
            jobs::table
                .filter(jobs::job_id.eq(job_id))
                .filter(jobs::user_id.eq(user.id)),
        )
        .set(finish)
        .execute(&mut connection)
        .context("Failed write finished job to database")?;

        Ok(())
    }

    async fn dismiss(&self, id: &str, user: &Self::User) -> anyhow::Result<Option<StatusInfo>> {
        let mut connection = self
            .connection
            .get()
            .context("could not get db connection from pool")?;

        let returned: Option<model::StatusInfo> = diesel::update(jobs::table)
            .filter(jobs::job_id.eq(id))
            .filter(jobs::user_id.eq(user.id))
            .set(DismissJob {
                status: StatusCode::Dismissed.into(),
                message: Some("Job dismissed by user"),
                updated: Utc::now(),
            })
            .returning(model::StatusInfo::as_returning())
            .get_result(&mut connection)
            .optional()
            .context("Failed to dismiss job in database")?;

        Ok(returned.map(Into::into))
    }

    async fn results(&self, id: &str, user: &Self::User) -> anyhow::Result<ProcessResult> {
        let mut connection = self
            .connection
            .get()
            .context("could not get db connection from pool")?;

        let results: Option<(Option<serde_json::Value>, model::Response)> = jobs::table
            .select((jobs::results, jobs::response))
            .filter(jobs::job_id.eq(id))
            .filter(jobs::user_id.eq(user.id))
            .first(&mut connection)
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
    use crate::{config::CONFIG, db::setup_db};
    use ogcapi::drivers::JobHandler as _;

    fn mock_db_pool() -> DbPool {
        setup_db(&CONFIG.database).unwrap()
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

    #[tokio::test]
    async fn test_register() {
        let pool = mock_db_pool();
        let handler = JobHandler { connection: pool };
        let user = mock_user();
        let status_info = mock_status_info("");
        let result = handler.register(&status_info, Response::Raw, &user).await;
        assert!(result.is_ok());
        let job_id = result.unwrap();
        assert!(!job_id.is_empty());
    }

    #[tokio::test]
    async fn test_update() {
        let pool = mock_db_pool();
        let handler = JobHandler { connection: pool };
        let user = mock_user();
        let status_info = mock_status_info("job1");
        // Register first
        handler
            .register(&status_info, Response::Raw, &user)
            .await
            .unwrap();
        // Update
        let mut updated_info = status_info.clone();
        updated_info.status = StatusCode::Running;
        let result = handler.update(&updated_info, &user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_status_list() {
        let pool = mock_db_pool();
        let handler = JobHandler { connection: pool };
        let user = mock_user();
        // Register a job
        let status_info = mock_status_info("job2");
        handler
            .register(&status_info, Response::Raw, &user)
            .await
            .unwrap();
        // List
        let result = handler.status_list(0, 10, &user).await;
        assert!(result.is_ok());
        let list = result.unwrap();
        assert!(!list.is_empty());
    }

    #[tokio::test]
    async fn test_status() {
        let pool = mock_db_pool();
        let handler = JobHandler { connection: pool };
        let user = mock_user();
        let status_info = mock_status_info("job3");
        handler
            .register(&status_info, Response::Raw, &user)
            .await
            .unwrap();
        let result = handler.status("job3", &user).await;
        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.is_some());
        assert_eq!(status.unwrap().job_id, "job3");
    }

    #[tokio::test]
    async fn test_finish() {
        let pool = mock_db_pool();
        let handler = JobHandler { connection: pool };
        let user = mock_user();
        let status_info = mock_status_info("job4");
        handler
            .register(&status_info, Response::Raw, &user)
            .await
            .unwrap();
        let result = handler
            .finish(
                "job4",
                &StatusCode::Successful,
                Some("done".to_string()),
                vec![],
                None,
                &user,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dismiss() {
        let pool = mock_db_pool();
        let handler = JobHandler { connection: pool };
        let user = mock_user();
        let status_info = mock_status_info("job5");
        handler
            .register(&status_info, Response::Raw, &user)
            .await
            .unwrap();
        let result = handler.dismiss("job5", &user).await;
        assert!(result.is_ok());
        let dismissed = result.unwrap();
        assert!(dismissed.is_some());
        assert_eq!(dismissed.unwrap().status, StatusCode::Dismissed);
    }

    #[tokio::test]
    async fn test_results_no_job() {
        let pool = mock_db_pool();
        let handler = JobHandler { connection: pool };
        let user = mock_user();
        let result = handler.results("no_such_job", &user).await;
        assert!(matches!(result.unwrap(), ProcessResult::NoSuchJob));
    }
}
