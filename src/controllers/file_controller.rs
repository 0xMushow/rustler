use axum::{extract::{Multipart, State}, response::IntoResponse, Json};
use std::sync::Arc;
use axum::extract::Path;
use axum::http::StatusCode;
use serde_json::json;
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

pub async fn view_codebase_handler(
    State(clients): State<Arc<Clients>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let file_service = FileService::new(clients);

    let output_dir = format!("./competitions/{}", name);

    match file_service.download_and_extract_zip(&name, &output_dir).await {
        Ok(files) => (StatusCode::OK, Json(json!({ "files": files }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}