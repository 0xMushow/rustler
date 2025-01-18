use std::sync::Arc;
use axum::{Router, routing::{post, get}};
use axum::extract::DefaultBodyLimit;
use crate::clients::clients::Clients;
use crate::controllers::file_controller::{upload_handler, view_codebase_handler};

/// Defines the file routes.
///
/// # Parameters
/// - `state`: The application clients.
///
/// # Returns
/// A Router containing the file routes.
/// Disable the default body limit for the `/upload` route to allow large file uploads.
///
pub fn file_routes(state: Arc<Clients>) -> Router {
    Router::new()
        .route("/upload", post(upload_handler)
            .layer(DefaultBodyLimit::disable())
            .with_state(state.clone()))
        .route("/view-codebase/{name}", get(view_codebase_handler)
            .with_state(state))
}