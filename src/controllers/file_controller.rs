use std::fs;
use axum::{extract::{Multipart, State}, response::IntoResponse, Json};
use std::sync::Arc;
use axum::extract::Path;
use axum::http::StatusCode;
use log::{error, info, warn};
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

/// Handles the view codebase request.
///
/// This function first checks if the requested codebase is already available locally,
/// then checks the Redis cache for the file. If the file is not found, it proceeds to
/// download and extract the archive. The extracted files are then cached in Redis.
///
/// # Parameters
/// - `State(clients)`: The application clients to interact with Redis, S3, and other services.
/// - `Path(name)`: The name of the codebase being requested.
///
pub async fn view_codebase_handler(
    State(clients): State<Arc<Clients>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let file_service = FileService::new(clients);
    let output_dir = format!("./competitions/{}", name);

    if fs::metadata(&output_dir).is_ok() {
        info!("File already exists locally: {}", name);

        match file_service.get_cached_file(&name).await {
            Ok(Some(cached_file)) => {
                info!("Returning cached file for: {}", name);
                (StatusCode::OK, Json(json!({ "file": cached_file }))).into_response()
            }
            Ok(None) => {
                warn!("File not found in cache for: {}", name);
                (StatusCode::OK, Json(json!({ "files": vec![output_dir] }))).into_response()
            },
            Err(_) => {
                error!("Failed to retrieve cached file for: {}", name);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Failed to retrieve cached file" }))).into_response()
            }
        }
    } else {
        match file_service.download_and_extract_archive(&name, &output_dir).await {
            Ok(files) => {
                info!("Successfully extracted files for: {}", name);

                if let Err(e) = file_service.cache_files(&name, &files).await {
                    error!("Error caching extracted files for {}: {}", name, e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response();
                }

                (StatusCode::OK, Json(json!({ "files": files }))).into_response()
            }
            Err(e) => {
                error!("Failed to extract files for {}: {}", name, e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))).into_response()
            }
        }
    }
}