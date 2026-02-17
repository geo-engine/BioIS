use biois::{CONFIG, server};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let service = server().await?;
    service.serve().await
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(CONFIG.logging.clone().into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();
}
