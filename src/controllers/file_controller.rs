use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
};
use std::sync::Arc;
use crate::clients::clients::Clients;
use crate::services::file_service::FileService;

/// Handles file uploads.
///
/// # Parameters
/// - `clients`: The application clients.
/// - `multipart`: The multipart request containing the file.
///
/// # Returns
/// The response to return to the client.
///
pub async fn upload_handler(
    State(clients): State<Arc<Clients>>,
    multipart: Multipart,
) -> impl IntoResponse {
    let file_service = FileService::new(clients);
    file_service.upload_file(multipart).await
}