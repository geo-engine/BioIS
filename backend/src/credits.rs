use crate::{
    CONFIG,
    db::{
        DbHandle,
        model::{ComputationId, Credits, TimestampMillis},
    },
    server::AppState,
    state::{CONTEXT, TaskLocalContext},
};
use anyhow::{Context, Result};
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{TimeZone, Utc};
use geoengine_api_client::{
    apis::{configuration::Configuration, user_api::computation_quota_handler},
    models::OperatorQuota,
};
use ogcapi::{
    services::{self as ogcapi_services},
    types::common::Exception,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::LazyLock};
use tokio::{sync::mpsc, time::sleep};
use tracing::{error, warn};
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
    add_credits_opt(db, None, computation_id, Some(credits)).await
}

/// Add credits used for a specific job to the database.
///
/// The job ID is retrieved from the task-local context, which must be set before calling this function.
pub async fn add_credits_pending(
    db: DbHandle,
    configuration: Configuration,
    computation_id: ComputationId,
) -> anyhow::Result<()> {
    add_credits_opt(db, Some(configuration), computation_id, None).await
}

async fn add_credits_opt(
    mut db: DbHandle,
    configuration: Option<Configuration>,
    computation_id: ComputationId,
    credits: Option<u64>,
) -> anyhow::Result<()> {
    let Some(job_id) = CONTEXT.job_id()? else {
        // anyhow::bail!("No job ID set in the task context"); // TODO: enforce this always
        warn!(
            "computationId"=%computation_id,
            "No job ID set in the task context, skipping adding credits"
        );
        return Ok(());
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

    if let Some(configuration) = configuration
        && computation_id.is_some()
    {
        enqueue_credits_lookup(configuration, db.clone(), job_id, computation_id);
    }

    Ok(())
}

static CREDITS_LOOKUP_QUEUE: LazyLock<mpsc::UnboundedSender<CreditsLookup>> = LazyLock::new(|| {
    let (tx, mut rx) = mpsc::unbounded_channel::<CreditsLookup>();
    let reentry_tx = tx.clone();
    tokio::spawn(async move {
        let mut pending_lookups = Vec::new();
        loop {
            // Process all available items
            while let Ok(mut lookup) = rx.try_recv() {
                // stop if the queue is empty
                match lookup.process().await {
                    Ok(CreditsLookupStatus::Finished) => {
                        // Credits lookup finished successfully, nothing to do
                    }
                    Ok(CreditsLookupStatus::Pending) => {
                        // Credits lookup is still pending, requeue the lookup for later processing
                        pending_lookups.push(lookup);
                    }
                    Err(e) => {
                        error!(
                            "Failed to process credits lookup for job {job_id} and computation ID {computation_id}: {e:?}",
                            job_id = &lookup.job_id,
                            computation_id = &lookup.computation_id
                        );
                        // TODO: Decide whether to requeue the lookup or not.
                        //       For now, we will not requeue it to avoid infinite error loops.
                    }
                }
            }

            // Move pending lookups back into the queue
            for lookup in pending_lookups.drain(..) {
                enqueue_credits_lookup_inner(&reentry_tx, lookup);
            }

            // Queue is empty, wait 1 minute before processing next batch
            sleep(CONFIG.credits.credits_lookup_interval()).await;
        }
    });
    tx
});

fn enqueue_credits_lookup_inner(tx: &mpsc::UnboundedSender<CreditsLookup>, lookup: CreditsLookup) {
    if let Err(e) = tx.send(lookup) {
        error!(
            "Failed to enqueue credits lookup for job {job_id} and computation ID {computation_id}: {e:?}",
            job_id = &e.0.job_id,
            computation_id = &e.0.computation_id
        );
    }
}

fn enqueue_credits_lookup(
    configuration: Configuration,
    db: DbHandle,
    job_id: Uuid,
    computation_id: ComputationId,
) {
    let lookup = CreditsLookup::new(configuration, db, job_id, computation_id);
    enqueue_credits_lookup_inner(&CREDITS_LOOKUP_QUEUE, lookup);
}

/// Data for looking up credits for a specific job and computation ID.
///
/// Geo Engine batches computation logging, so we need to store the computation ID to be able to look up the credits
/// used for a specific job, later.
struct CreditsLookup {
    configuration: Configuration,
    db: DbHandle,

    job_id: Uuid,
    computation_id: ComputationId,

    quotas: Option<Vec<geoengine_api_client::models::OperatorQuota>>,
}

enum CreditsLookupStatus {
    Pending,
    Finished,
}

impl CreditsLookup {
    fn new(
        configuration: Configuration,
        db: DbHandle,
        job_id: Uuid,
        computation_id: ComputationId,
    ) -> Self {
        Self {
            configuration,
            db,
            job_id,
            computation_id,
            quotas: None,
        }
    }

    /// Process the credits lookup by querying the Geo Engine API and storing the result in the database.
    async fn process(&mut self) -> Result<CreditsLookupStatus> {
        let quotas =
            computation_quota_handler(&self.configuration, &self.computation_id.to_string())
                .await?;

        if quotas.is_empty() {
            // Quotas are empty, the computation credits are still missing, so we need to wait for the next batch
            return Ok(CreditsLookupStatus::Pending);
        }

        let Some(stored_quotas) = &self.quotas else {
            // We got quotas, but we need to make sure that the result is consistent
            self.quotas = Some(quotas);
            return Ok(CreditsLookupStatus::Pending);
        };

        if stored_quotas != &quotas {
            // Quotas have changed, we need to wait for the next batch
            self.quotas = Some(quotas);
            return Ok(CreditsLookupStatus::Pending);
        }

        self.update_credits().await?;

        Ok(CreditsLookupStatus::Finished)
    }

    async fn update_credits(&self) -> Result<()> {
        fn quotas_sum<'q>(quotas: impl IntoIterator<Item = &'q OperatorQuota>) -> u64 {
            quotas.into_iter().map(|q| q.count as u64).sum()
        }

        let credits = self.quotas.as_ref().map(quotas_sum);

        toasty::update!(Credits:: filter_by_job_id(self.job_id) {
            credits,
        })
        .exec(&mut self.db.db())
        .await
        .context("Failed to update credits in database")
    }
}
