use crate::{
    CONFIG,
    db::{
        DbHandle,
        model::{ComputationId, Credits, TimestampMillis},
    },
    state::{CONTEXT, TaskLocalContext},
};
use anyhow::{Context, Result};
use chrono::Utc;
use geoengine_api_client::{
    apis::{configuration::Configuration, user_api::computation_quota_handler},
    models::OperatorQuota,
};
use tokio::{task::JoinHandle, time::sleep};
use tracing::{error, instrument, warn};
use uuid::Uuid;

const LOOKUP_RETRY_THRESHOLD: i64 = 5;

/// Add credits used for a specific job to the database.
///
/// The job ID is retrieved from the task-local context, which must be set before calling this function.
pub async fn add_credits_used(
    db: DbHandle,
    computation_id: ComputationId,
    credits: u64,
) -> anyhow::Result<()> {
    add_credits_used_opt(db, None, computation_id, None, credits).await
}

/// Add credits used for a specific job to the database.
///
/// The job ID is retrieved from the task-local context, which must be set before calling this function.
pub async fn add_credits_used_pending(
    db: DbHandle,
    configuration: Configuration,
    computation_id: ComputationId,
) -> anyhow::Result<()> {
    add_credits_used_opt(db, Some(configuration), computation_id, None, 0).await
}

async fn add_credits_used_opt(
    mut db: DbHandle,
    configuration: Option<Configuration>,
    computation_id: ComputationId,
    geoengine_credits: Option<u64>,
    biois_credits: u64,
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
        timestamp: TimestampMillis::from(Utc::now()),
        geoengine_credits,
        biois_credits,
        pending: configuration.is_some() && computation_id.is_some(),
        configuration: configuration.map(Into::into),
    })
    .exec(db.as_mut())
    .await
    .context("failed to insert credits into database")?;

    Ok(())
}

/// Starts a task for looking up processing credits repeatedly in the background
pub fn start_lookup_task(mut db: DbHandle) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            process_batch(&mut db).await;
            wait_for_next_lookup_interval().await;
        }
    })
}

/// Test version of the lookup task that only runs once and then exits, for use in unit tests.
#[cfg(test)]
pub fn run_lookup_task_once(mut db: DbHandle) -> JoinHandle<()> {
    tokio::spawn(async move {
        process_batch(&mut db).await;
    })
}

/// Process a batch of credits that are pending lookup from the Geo Engine API.
async fn process_batch(db: &mut DbHandle) {
    for credits in credits_to_lookup(db).await.unwrap_or_default() {
        let Err(error) = process_credit(db, &credits).await else {
            // Successfully processed the credits, continue to the next one
            continue;
        };

        error!(
            "Failed to process credits for job_id={job_id} computation_id={computation_id}: {error}",
            job_id = credits.job_id,
            computation_id = credits.computation_id,
        );

        if let Err(error) = update_credits_with_error(
            db,
            credits.job_id,
            credits.computation_id,
            credits.errors,
            error.to_string(),
        )
        .await
        {
            error!(
                "Failed to update credits with error for job_id={job_id} computation_id={computation_id}: {error}",
                job_id = credits.job_id,
                computation_id = credits.computation_id,
            );
        }
    }
}

/// Processing was done, wait <INTERVAL> before processing next batch
async fn wait_for_next_lookup_interval() {
    sleep(CONFIG.credits.credits_lookup_interval()).await;
}

async fn credits_to_lookup(db: &mut DbHandle) -> Result<Vec<Credits>> {
    let fields = Credits::fields();
    let is_pending = fields.pending().eq(true);
    let is_retryable = fields.errors().len().le(LOOKUP_RETRY_THRESHOLD);
    let credits_list = Credits::filter(is_pending.and(is_retryable))
        .exec(db.as_mut())
        .await?;

    Ok(credits_list)
}

/// Process the credits lookup by querying the Geo Engine API and storing the result in the database.
#[instrument(skip(db), level = "debug", fields(job_id = %credits.job_id, computation_id = %credits.computation_id), ret)]
async fn process_credit(db: &mut DbHandle, credits: &Credits) -> Result<()> {
    let quotas = computation_quota_handler(
        &credits
            .configuration
            .clone()
            .context("Missing API configuration for credits")?
            .into(),
        &credits.computation_id.to_string(),
    )
    .await?;

    if quotas.is_empty() {
        // Quotas are empty, the computation credits are still missing, so we need to wait for the next batch
        return Ok(());
    }
    let quotas_sum = quotas_sum(&quotas);

    let Some(stored_quota_sum) = credits.geoengine_credits else {
        // We got quotas, but we need to make sure that the result is consistent
        // so we store the quotas and wait for the next batch to make sure that the result is consistent
        return update_credits(db, credits.job_id, credits.computation_id, quotas_sum, true).await;
    };

    // If the results are not stable, the state is still pending, so we need to wait for the next batch
    let pending = stored_quota_sum != quotas_sum;

    // Quotas are available, we can store them in the database
    update_credits(
        db,
        credits.job_id,
        credits.computation_id,
        quotas_sum,
        pending,
    )
    .await
}

/// Calculate the sum of credits from the list of operator quotas.
fn quotas_sum<'q>(quotas: impl IntoIterator<Item = &'q OperatorQuota>) -> u64 {
    quotas.into_iter().map(|q| q.count.cast_unsigned()).sum()
}

async fn update_credits(
    db: &mut DbHandle,
    job_id: Uuid,
    computation_id: ComputationId,
    quota_sum: u64,
    pending: bool,
) -> Result<()> {
    toasty::update!(Credits::filter_by_job_id_and_computation_id(job_id, computation_id) {
        geoengine_credits: quota_sum,
        pending: pending,
        errors: Vec::<String>::new(), // reset errors on successful update
    })
    .exec(db.as_mut())
    .await
    .context("Failed to update credits in database")
}

async fn update_credits_with_error(
    db: &mut DbHandle,
    job_id: Uuid,
    computation_id: ComputationId,
    previous_errors: Vec<String>,
    error: String,
) -> Result<()> {
    let mut errors = previous_errors;
    errors.push(error);

    toasty::update!(Credits::filter_by_job_id_and_computation_id(job_id, computation_id) {
        errors,
    })
    .exec(db.as_mut())
    .await
    .context("Failed to update credits in database")
}
