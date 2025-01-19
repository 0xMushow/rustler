use std::path::{Path as FilePath, PathBuf};
use std::{fs, io};
use axum::{extract::{Multipart, State}, response::IntoResponse, Json};
use std::sync::Arc;
use axum::extract::Path;
use axum::http::StatusCode;
use indexmap::IndexMap;
use log::{error, info, warn};
use serde_json::{json, Value};
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

/// Recursively traverses a directory and returns its structure as a JSON-compatible `Value`.
/// The structure is represented as an array of objects, where each object represents a file or folder.
/// Each object contains the following keys:
/// - `name`: The name of the file or folder.
/// - `type`: The type of the item, either "file" or "folder".
/// - `children`: An array of objects representing the children of the folder.
///
/// # Parameters
/// - `path`: The path to the directory to traverse.
///
/// # Returns
/// A `Value` representing the directory structure.
fn traverse_directory(path: &FilePath) -> Result<Vec<Value>, io::Error> {
    let mut items = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name().to_string_lossy().to_string();

        if entry_path.is_dir() {
            let mut folder = IndexMap::new(); // Use IndexMap to preserve insertion order

            folder.insert("name".to_string(), Value::String(entry_name.clone()));
            folder.insert("type".to_string(), Value::String("folder".to_string()));

            let children = traverse_directory(&entry_path)?;
            folder.insert("children".to_string(), Value::Array(children));

            let folder_value = Value::Object(folder.into_iter().collect());

            items.push(folder_value);
        } else {
            let mut file = IndexMap::new(); // Use IndexMap to preserve insertion order

            file.insert("name".to_string(), Value::String(entry_name));
            file.insert("type".to_string(), Value::String("file".to_string()));

            let file_value = Value::Object(file.into_iter().collect());

            items.push(file_value);
        }
    }

    Ok(items)
}

/// Axum handler to view the codebase structure as JSON.
///
/// # Parameters
/// - `Path(repo_name)`: The name of the repository to generate the codebase JSON for.
///
/// # Returns
/// The JSON response containing the codebase structure.
pub async fn generate_codebase_json(Path(repo_name): Path<String>) -> Result<Json<Value>, (StatusCode, String)> {
    let base_path = PathBuf::from("competitions");
    let repo_path = base_path.join(&repo_name);

    if !repo_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Repository '{}' not found in 'competitions' directory", repo_name),
        ));
    }

    let structure = match traverse_directory(&repo_path) {
        Ok(s) => s,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to traverse directory: {}", e),
            ));
        }
    };

    Ok(Json(json!({
        "status": "success",
        "message": "Codebase JSON generated successfully",
        "data": structure,
    })))
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
                file_service.cache_files(&name, &vec![output_dir.clone()]).await.unwrap();
                info!("Cached file for: {}", name);
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