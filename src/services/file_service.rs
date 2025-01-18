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
use crate::utils::file_utils::FileValidator;

/// A service to handle file-related operations.
/// This service is used to upload files to S3.
pub struct FileService {
    clients: Arc<Clients>,
    validator: FileValidator,
}

impl FileService {
    /// Creates a new instance of `FileService`.
    pub fn new(clients: Arc<Clients>) -> Self {
        info!("FileService initialized");
        Self {
            clients,
            validator: FileValidator::new(),
        }
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

        let file_name = field.file_name().unwrap_or("").to_string();
        let extension = file_name.split('.').last().unwrap_or("").to_lowercase();

        let file_type = match self.validator.find_file_type_by_extension(&extension) {
            Some(file_type) => file_type,
            None => {
                warn!("Unsupported file extension: {}", extension);
                return self.error_response(StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported file extension");
            }
        };

        match self.validator.validate_file(&file_type.name, &mut field).await {
            Ok(buffer) => {
                match self.clients.get_s3_client().upload_file(&file_name, &buffer).await {
                    Ok(_) => {
                        info!(
                        "Successfully uploaded file to S3: '{}'. Size: {} bytes",
                        file_name,
                        buffer.len()
                    );
                        self.success_response(file_name, buffer.len())
                    }
                    Err(e) => {
                        error!("Error uploading file to S3: '{}'. Error: {:?}", file_name, e);
                        self.error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            &format!("Failed to upload file to S3: {:?}", e),
                        )
                    }
                }
            }
            Err(validation_error) => {
                warn!(
                "File validation failed for '{}': {}",
                file_name, validation_error.message
            );
                self.error_response(validation_error.code, &validation_error.message)
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
