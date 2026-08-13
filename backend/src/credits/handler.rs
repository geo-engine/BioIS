use crate::{
    db::model::{Credits, TimestampMillis},
    server::AppState,
    state::{CONTEXT, TaskLocalContext},
};
use anyhow::Context;
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{TimeZone, Utc};
use ogcapi::{
    services::{self as ogcapi_services},
    types::common::Exception,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_credits))
}

/// Returns the user's credits.
#[utoipa::path(get, path = "/{year}/{month}", tag = "User",
    params(GetCreditsParams),
    responses(
        (
            status = OK,
            description = "The user's credits for the specified month.",
            body = GetCreditsResponse
        ),
        (
            status = INTERNAL_SERVER_ERROR,
            description = "A server error occurred.", 
            body = Exception,
            example = json!(Exception::new_from_status(500))
        )
    )
)]
pub async fn get_credits(
    Path(GetCreditsParams { year, month }): Path<GetCreditsParams>,
    State(app_state): State<AppState>,
) -> ogcapi_services::Result<Json<GetCreditsResponse>> {
    let user_id = CONTEXT.user_id()?;
    let timestamp_start_inclusive = Utc
        .with_ymd_and_hms(i32::from(year), u32::from(month), 1, 0, 0, 0)
        .earliest()
        .context("invalid start timestamp")?;
    let timestamp_end_exclusive: TimestampMillis = timestamp_start_inclusive
        .checked_add_months(chrono::Months::new(1))
        .context("invalid end timestamp")?
        .into();
    let timestamp_start_inclusive: TimestampMillis = timestamp_start_inclusive.into();

    let fields = Credits::fields();

    let credits = Credits::filter(
        fields
            .timestamp()
            .ge(timestamp_start_inclusive)
            .and(fields.timestamp().lt(timestamp_end_exclusive))
            .and(fields.job().user_id().eq(user_id)),
    )
    .exec(&mut app_state.db.db())
    .await
    .context("failed to execute credits query")?;

    let credits_per_job = Vec::<CreditsForJob>::from_credits(&credits);

    Ok(Json(GetCreditsResponse {
        year,
        month,
        credits_used: credits_per_job.sum(),
        details: credits_per_job,
    }))
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct GetCreditsParams {
    pub year: u16,
    pub month: u8,
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetCreditsResponse {
    pub year: u16,
    pub month: u8,
    pub credits_used: u64,
    pub details: Vec<CreditsForJob>, // Add this field to include the details of credits
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreditsForJob {
    pub job_id: Uuid,
    pub credits_used: u64,
}

/// A trait to sum credits used for a list of credits.
///
/// TODO: Use `toasty`'s aggregate queries (<https://github.com/tokio-rs/toasty/issues/421>) once available.
trait SumCredits {
    fn sum(&self) -> u64;

    fn from_credits(credits_list: &[Credits]) -> Self;
}

impl SumCredits for Vec<CreditsForJob> {
    fn sum(&self) -> u64 {
        self.iter().map(|c| c.credits_used).sum()
    }

    fn from_credits(credits_list: &[Credits]) -> Self {
        // Since UUID v7 is time-ordered, we can use a BTreeMap to aggregate credits by job_id.
        let mut map = BTreeMap::<Uuid, u64>::new();
        for c in credits_list {
            let entry = map.entry(c.job_id).or_insert(0);
            *entry += c.credits.unwrap_or(0);
        }
        map.into_iter()
            .map(|(job_id, credits_used)| CreditsForJob {
                job_id,
                credits_used,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::User,
        credits::add_credits_used,
        db::model::{ComputationId, JobType, StatusCode},
        jobs::JobHandler,
        server::AppState,
        state::TaskContext,
        util::Secret,
    };
    use axum::extract::Path;
    use chrono::Datelike;
    use ogcapi::{
        drivers::JobHandler as _,
        types::processes::{Response as ApiResponse, StatusInfo},
    };
    use pretty_assertions::assert_eq;

    fn mock_user() -> User {
        User {
            id: Uuid::from_u128(0xabcd_efab_cdef_abcd_efab_cdef_abcd_efab),
            session_token: Secret(Uuid::from_u128(0x1234_5678_90ab_cdef_1234_5678_90ab_cdef)),
        }
    }

    fn year_and_month() -> (u16, u8) {
        let current_date = Utc::now();
        (
            u16::try_from(current_date.year()).unwrap(),
            u8::try_from(current_date.month()).unwrap(),
        )
    }

    #[crate::test(task_context = TaskContext::new(mock_user()))]
    async fn it_returns_empty_list_when_no_credits_for_month(db: DbHandle) {
        let app_state = AppState {
            db,
            api_config: crate::CONFIG.geoengine.api_config(None),
        };

        let Json(result) = get_credits(
            Path(GetCreditsParams {
                year: 2024,
                month: 1,
            }),
            axum::extract::State(app_state),
        )
        .await
        .unwrap();

        assert_eq!(
            result,
            GetCreditsResponse {
                year: 2024,
                month: 1,
                credits_used: 0,
                details: vec![],
            }
        );
    }

    #[crate::test(task_context = TaskContext::new(mock_user()))]
    async fn it_returns_one_credit_per_job_with_different_computation_ids(db: DbHandle) {
        let app_state = AppState {
            db: db.clone(),
            api_config: crate::CONFIG.geoengine.api_config(None),
        };

        let job_handler = JobHandler::new(db.clone()).await.unwrap();

        let job_id_1 = job_handler
            .register(
                &StatusInfo {
                    r#type: JobType::Process.into(),
                    job_id: String::new(),
                    status: StatusCode::Successful.into(),
                    ..Default::default()
                },
                ApiResponse::Document,
            )
            .await
            .unwrap()
            .parse()
            .unwrap();

        add_credits_used(
            db.clone(),
            ComputationId::some(Uuid::from_u128(0x1111_1111_1111_1111_1111_1111_1111_1111)),
            100,
        )
        .await
        .unwrap();

        let job_id_2 = job_handler
            .register(
                &StatusInfo {
                    r#type: JobType::Process.into(),
                    job_id: String::new(),
                    status: StatusCode::Successful.into(),
                    ..Default::default()
                },
                ApiResponse::Document,
            )
            .await
            .unwrap()
            .parse()
            .unwrap();

        add_credits_used(db.clone(), ComputationId::none(), 50)
            .await
            .unwrap();

        let (year, month) = year_and_month();
        let Json(result) = get_credits(
            Path(GetCreditsParams { year, month }),
            axum::extract::State(app_state),
        )
        .await
        .unwrap();

        assert_eq!(
            result,
            GetCreditsResponse {
                year,
                month,
                credits_used: 150,
                details: vec![
                    CreditsForJob {
                        job_id: job_id_1,
                        credits_used: 100
                    },
                    CreditsForJob {
                        job_id: job_id_2,
                        credits_used: 50
                    }
                ],
            }
        );
    }

    #[crate::test(task_context = TaskContext::new(User {
        id: Uuid::new_v4(),
        session_token: Secret(Uuid::new_v4()),
    }))]
    async fn it_aggregates_multiple_credits_for_one_job(db: DbHandle) {
        let app_state = AppState {
            db: db.clone(),
            api_config: crate::CONFIG.geoengine.api_config(None),
        };

        let job_handler = JobHandler::new(db.clone()).await.unwrap();

        let job_id_1 = job_handler
            .register(
                &StatusInfo {
                    r#type: JobType::Process.into(),
                    job_id: String::new(),
                    status: StatusCode::Successful.into(),
                    ..Default::default()
                },
                ApiResponse::Document,
            )
            .await
            .unwrap()
            .parse()
            .unwrap();

        let computation_id_1 =
            ComputationId::some(Uuid::from_u128(0x1111_1111_1111_1111_1111_1111_1111_1111));
        add_credits_used(db.clone(), computation_id_1, 100)
            .await
            .unwrap();

        let computation_id_2 =
            ComputationId::some(Uuid::from_u128(0x2222_2222_2222_2222_2222_2222_2222_2222));
        add_credits_used(db.clone(), computation_id_2, 75)
            .await
            .unwrap();

        let (year, month) = year_and_month();
        let Json(result) = get_credits(
            Path(GetCreditsParams { year, month }),
            axum::extract::State(app_state),
        )
        .await
        .unwrap();

        assert_eq!(
            result,
            GetCreditsResponse {
                year,
                month,
                credits_used: 175,
                details: vec![CreditsForJob {
                    job_id: job_id_1,
                    credits_used: 175
                }],
            }
        );
    }
}
