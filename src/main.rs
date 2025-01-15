mod config;
mod error;
mod clients;

use log::{error, info};
use config::AppConfig;

use anyhow::Result;
use crate::clients::clients::Clients;

async fn run() -> Result<()> {
    env_logger::init();

    let config = AppConfig::from_env()?;
    let clients = Clients::new(&config).await?;
    clients.test_connections().await?;

    info!("All connections successful!");
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("Application error: {}", e);
    }
}