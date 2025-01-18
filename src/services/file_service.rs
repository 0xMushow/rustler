use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use axum::response::Response;
use log::{error, info, warn};
use crate::clients::clients::Clients;

/// The maximum allowed file size in bytes.
/// This is set to 100 MB.
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

/// A service to handle file-related operations.
/// This service is used to upload files to S3.
pub struct FileService {
    clients: Arc<Clients>,
}

impl FileService {
    /// Creates a new instance of `FileService`.
    pub fn new(clients: Arc<Clients>) -> Self {
        info!("FileService initialized");
        Self { clients }
    }

    /// Handles file uploads.
    /// This function reads the file content from the multipart request,
    /// validates the file type and size, and uploads the file to S3.
    /// If successful, it returns a success response with the file details.
    /// If an error occurs, it returns an error response.
    ///
    /// # Parameters
    /// - `multipart`: The multipart request containing the file.
    ///
    /// # Returns
    /// The response to return to the client.
    pub async fn upload_file(&self, mut multipart: Multipart) -> Response {
        let mut field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => {
                warn!("No file provided in the request");
                return self.error_response(StatusCode::BAD_REQUEST, "No file provided");
            }
            Err(e) => {
                error!("Failed to parse multipart data: {:?}", e);
                return self.error_response(StatusCode::BAD_REQUEST, "Failed to parse multipart data");
            }
        };

        let file_name = match field.file_name() {
            Some(name) if name.to_lowercase().ends_with(".zip") => name.to_string(),
            Some(name) => {
                warn!(
                    "Invalid file type provided. File name: '{}', Content-Type: '{}'",
                    name,
                    field.content_type().unwrap_or("unknown")
                );
                return self.error_response(StatusCode::UNSUPPORTED_MEDIA_TYPE, "Only .zip files are allowed");
            }
            None => {
                warn!(
                    "No filename provided. Content-Type: '{}'",
                    field.content_type().unwrap_or("unknown")
                );
                return self.error_response(StatusCode::BAD_REQUEST, "No filename provided");
            }
        };

        let content_type = field.content_type().unwrap_or("").to_string();
        if content_type != "application/zip" {
            warn!(
                "Unsupported content type for file: '{}'. Content-Type: '{}'",
                file_name, content_type
            );
            return self.error_response(StatusCode::UNSUPPORTED_MEDIA_TYPE, "Only .zip files are allowed");
        }

        let mut total_bytes = 0;
        let mut buffer = Vec::new();

        info!("Starting to stream file content...");

        while let Some(chunk) = field.chunk().await.transpose() {
            match chunk {
                Ok(data) => {
                    total_bytes += data.len();

                    if total_bytes > MAX_FILE_SIZE {
                        warn!(
                            "File exceeds maximum allowed size: {} bytes. File name: '{}', Content-Type: '{}'",
                            MAX_FILE_SIZE, file_name, content_type
                        );
                        return self.error_response(StatusCode::PAYLOAD_TOO_LARGE, "File exceeds maximum allowed size");
                    }

                    buffer.extend_from_slice(&data);
                }
                Err(e) => {
                    error!("Error reading chunk for file: '{}'. Error: {:?}", file_name, e);
                    return self.error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read chunk: {:?}", e));
                }
            }
        }

        match self.clients.get_s3_client().upload_file(&file_name, &buffer).await {
            Ok(_) => {
                info!(
                    "Successfully uploaded file to S3: '{}'. Size: {} bytes",
                    file_name, total_bytes
                );
                self.success_response(file_name, total_bytes)
            }
            Err(e) => {
                error!("Error uploading file to S3: '{}'. Error: {:?}", file_name, e);
                self.error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to upload file to S3: {:?}", e))
            }
        }
    }

    /// Helper function to create an error response.
    ///
    /// # Parameters
    /// - `status_code`: The HTTP status code.
    /// - `message`: The error message.
    ///
    /// # Returns
    /// The response to return to the client.
    fn error_response(&self, status_code: StatusCode, message: &str) -> Response {
        error!("Returning error response: {} - {}", status_code, message);
        (status_code, Json(json!({ "error": message }))).into_response()
    }

    /// Helper function to create a success response.
    ///
    /// # Parameters
    /// - `file_name`: The name of the uploaded file.
    /// - `size`: The size of the uploaded file.
    ///
    /// # Returns
    /// The response to return to the client.
    fn success_response(&self, file_name: String, size: usize) -> Response {
        info!("Returning success response for file: {} ({} bytes)", file_name, size);
        (
            StatusCode::OK,
            Json(json!({
                "message": "File uploaded successfully",
                "file_name": file_name,
                "size": size
            })),
        )
            .into_response()
    }
}
