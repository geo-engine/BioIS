use anyhow::Result;
use biois::{CONFIG, db::DbHandle};
use toasty_cli::{Config, ToastyCli};

#[tokio::main]
async fn main() -> Result<()> {
    let toasty_config = Config::load()?;

    let db = DbHandle::from_config(&CONFIG.database).await?;

    let cli = ToastyCli::with_config(db.db(), toasty_config);
    cli.parse_and_run().await?;

    Ok(())
}
