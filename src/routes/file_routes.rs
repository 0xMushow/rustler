use std::sync::Arc;
use axum::{Router, routing::post};
use axum::extract::DefaultBodyLimit;
use crate::clients::clients::Clients;
use crate::controllers::file_controller::upload_handler;

/// Defines the file routes.
///
/// # Parameters
/// - `state`: The application clients.
///
/// # Returns
/// A Router containing the file routes.
/// Disable the default body limit to allow large file uploads.
///
pub fn file_routes(state: Arc<Clients>) -> Router {
    Router::new().route("/upload", post(upload_handler)
        .layer(DefaultBodyLimit::disable())
        .with_state(state))
}
