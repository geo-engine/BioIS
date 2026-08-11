use crate::{
    db::{
        DbHandle,
        model::{ComputationId, Credits, TimestampMillis},
    },
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
use std::collections::HashMap;
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

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetCreditsResponse {
    pub year: u16,
    pub month: u8,
    pub credits_used: u32,
    pub details: Vec<CreditsForJob>, // Add this field to include the details of credits
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreditsForJob {
    pub job_id: Uuid,
    pub credits_used: u32,
}

/// A trait to sum credits used for a list of credits.
///
/// TODO: Use `toasty`'s aggregate queries (<https://github.com/tokio-rs/toasty/issues/421>) once available.
trait SumCredits {
    fn sum(&self) -> u32;

    fn from_credits(credits_list: &[Credits]) -> Self;
}

impl SumCredits for Vec<CreditsForJob> {
    fn sum(&self) -> u32 {
        self.iter().map(|c| c.credits_used).sum()
    }

    fn from_credits(credits_list: &[Credits]) -> Self {
        let mut map: HashMap<Uuid, u32> = HashMap::new();
        for c in credits_list {
            let entry = map.entry(c.job_id).or_insert(0);
            *entry += c.credits.unwrap_or(0) as u32;
        }
        map.into_iter()
            .map(|(job_id, credits_used)| CreditsForJob {
                job_id,
                credits_used,
            })
            .collect()
    }
}

/// Add credits used for a specific job to the database.
///
/// The job ID is retrieved from the task-local context, which must be set before calling this function.
pub async fn add_credits_used(
    db: DbHandle,
    computation_id: ComputationId,
    credits: u64,
) -> anyhow::Result<()> {
    add_credits_opt(db, computation_id, Some(credits)).await
}

/// Add credits used for a specific job to the database.
///
/// The job ID is retrieved from the task-local context, which must be set before calling this function.
pub async fn add_credits_pending(
    db: DbHandle,
    computation_id: ComputationId,
) -> anyhow::Result<()> {
    add_credits_opt(db, computation_id, None).await
}

async fn add_credits_opt(
    mut db: DbHandle,
    computation_id: ComputationId,
    credits: Option<u64>,
) -> anyhow::Result<()> {
    let Some(job_id) = CONTEXT.job_id()? else {
        anyhow::bail!("No job ID set in the task context");
    };

    toasty::create!(Credits {
        job_id,
        computation_id,
        credits,
        timestamp: TimestampMillis::from(Utc::now()),
    })
    .exec(db.as_mut())
    .await
    .context("failed to insert credits into database")?;

    Ok(())
}
