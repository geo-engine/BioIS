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
use std::sync::LazyLock;
use tokio::{sync::mpsc, time::sleep};
use tracing::{error, instrument, warn};
use uuid::Uuid;

/// A queue for processing credits lookups in the background.
///
/// TODO: Load pending credits lookups from the database on startup and enqueue them for processing.
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

fn enqueue_credits_lookup_inner(tx: &mpsc::UnboundedSender<CreditsLookup>, lookup: CreditsLookup) {
    if let Err(e) = tx.send(lookup) {
        error!(
            "Failed to enqueue credits lookup for job {job_id} and computation ID {computation_id}: {e:?}",
            job_id = &e.0.job_id,
            computation_id = &e.0.computation_id
        );
    }
}

#[instrument(skip(configuration, db), level = "debug")]
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
#[derive(Debug)]
struct CreditsLookup {
    configuration: Configuration,
    db: DbHandle,

    job_id: Uuid,
    computation_id: ComputationId,

    quotas: Option<Vec<geoengine_api_client::models::OperatorQuota>>,
}

#[derive(Debug)]
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
    #[instrument(skip(self), level = "debug", fields(job_id = %self.job_id, computation_id = %self.computation_id), ret)]
    async fn process(&mut self) -> Result<CreditsLookupStatus> {
        let quotas =
            computation_quota_handler(&self.configuration, &self.computation_id.to_string())
                .await?;

        if quotas.is_empty() {
            // Quotas are empty, the computation credits are still missing, so we need to wait for the next batch
            return Ok(CreditsLookupStatus::Pending);
        }

        // Quotas are available, we can store them in the database
        self.update_credits().await?;

        let Some(stored_quotas) = &self.quotas else {
            // We got quotas, but we need to make sure that the result is consistent
            // so we store the quotas and wait for the next batch to make sure that the result is consistent
            self.quotas = Some(quotas);
            return Ok(CreditsLookupStatus::Pending);
        };

        if stored_quotas != &quotas {
            // Quotas have changed, we need to wait for the next batch
            self.quotas = Some(quotas);
            return Ok(CreditsLookupStatus::Pending);
        }

        Ok(CreditsLookupStatus::Finished)
    }

    async fn update_credits(&self) -> Result<()> {
        fn quotas_sum<'q>(quotas: impl IntoIterator<Item = &'q OperatorQuota>) -> u64 {
            quotas.into_iter().map(|q| q.count.cast_unsigned()).sum()
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
