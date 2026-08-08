use biois::{CONFIG, server, setup_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _tracing_guard = setup_tracing(CONFIG.logging.clone().into());

    let service = server().await?;
    service.serve().await
}
