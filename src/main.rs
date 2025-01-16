//! The main entry point for the Rustler application.
//!
//! This module initializes the application, loads configuration, and tests connections
//! to external services (AWS S3, PostgreSQL, Redis). It also handles errors and logs
//! application events.

mod config;
mod error;
mod clients;

use std::sync::Arc;
use log::{error, info};
use config::AppConfig;

use anyhow::{Context, Result};
use axum::{serve, Extension, Router};
use tokio::net::TcpListener;
use crate::clients::clients::Clients;

/// The main application logic.
///
/// This function initializes the logger, loads the application configuration,
/// creates clients for external services, and tests their connections.
///
/// # Returns
/// - `Ok(())`: If all connections are successful.
/// - `Err(anyhow::Error)`: If any step fails.
async fn run() -> Result<()> {
    let config = AppConfig::from_env().context("Failed to load app configuration")?;
    info!("App configuration loaded successfully");

    let clients = Clients::new(&config)
        .await
        .context("Failed to initialize clients")?;
    info!("Clients initialized successfully");

    clients.test_connections().await.context("Failed to connect to external services")?;
    info!("Successfully connected to all external services");

    let app_state = Arc::new(clients);
    run_server(app_state).await;

    Ok(())
}

/// Starts the Axum server.
///
/// # Arguments
/// - `state`: A shared state containing the application clients.
///
async fn run_server(state: Arc<Clients>) {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server running on http://0.0.0.0:3000");

    let app = Router::new().layer(Extension(state.clone()));

    serve(listener, app).await.unwrap();
}

/// The entry point of the application.
///
/// This function initializes the Tokio runtime and runs the main application logic.
/// If an error occurs, it logs the error and exits the application.
#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(e) = run().await {
        error!("Application error: {}", e);
    }
}