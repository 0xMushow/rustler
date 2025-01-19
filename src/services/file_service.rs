use std::fs::{create_dir_all, File};
use std::{fs, io};
use std::io::{copy, Write};
use std::path::Path;
use std::process::Command;
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
use zip::ZipArchive;
use crate::clients::clients::Clients;
use crate::error::AppError;
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
        let extension = if file_name.ends_with(".tar.gz") {
            "tar.gz".to_string()
        } else {
            file_name.split('.').last().unwrap_or("").to_lowercase()
        };

        let file_type = match self.validator.find_file_type_by_extension(&extension) {
            Some(file_type) => {
                file_type
            },
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

    /// Downloads and extracts a ZIP or tar.gz file from S3.
    ///
    /// # Parameters
    /// - `s3_key`: The S3 key of the ZIP or tar.gz file.
    /// - `output_dir`: The directory where the file will be extracted.
    ///
    /// # Returns
    /// The response to return to the client.
    pub async fn download_and_extract_zip_or_tar(
        &self,
        s3_key: &str,
        output_dir: &str,
    ) -> Result<Vec<String>, AppError> {
        if s3_key.ends_with(".zip") {
            self.download_and_extract_zip(s3_key, output_dir).await
        } else if s3_key.ends_with(".tar.gz") {
            self.download_and_extract_tar_gz(s3_key, output_dir).await
        } else {
            Err(AppError::ValidationError("Unsupported file type".to_string()))
        }
    }


    /// Downloads and extracts a ZIP file from S3.
    ///
    /// # Parameters
    /// - `s3_key`: The S3 key of the ZIP file.
    /// - `output_dir`: The directory where the ZIP file will be extracted.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: A list of file paths extracted from the ZIP file.
    /// - `Err(AppError)`: An error if the download or extraction fails.
    pub async fn download_and_extract_zip(
        &self,
        s3_key: &str,
        output_dir: &str,
    ) -> Result<Vec<String>, AppError> {
        info!("Starting download and extraction of ZIP file for S3 key: {}", s3_key);

        // Download the ZIP file from S3
        let zip_data = match self.clients.get_s3_client().download_file(s3_key).await {
            Ok(data) => {
                info!("Successfully downloaded ZIP file from S3: {}", s3_key);
                data
            }
            Err(e) => {
                error!("Failed to download ZIP file from S3: {}. Error: {:?}", s3_key, e);
                return Err(e.into());
            }
        };

        // Create the output directory if it doesn't exist
        if let Err(e) = create_dir_all(output_dir) {
            error!("Failed to create output directory: {}. Error: {:?}", output_dir, e);
            return Err(e.into());
        }
        info!("Created or verified output directory: {}", output_dir);

        // Save the ZIP file locally
        let zip_path = Path::new(output_dir).join("temp.zip");
        let mut file = match File::create(&zip_path) {
            Ok(file) => {
                info!("Temporary ZIP file created at: {:?}", zip_path);
                file
            }
            Err(e) => {
                error!("Failed to create temporary ZIP file: {:?}. Error: {:?}", zip_path, e);
                return Err(e.into());
            }
        };

        if let Err(e) = file.write_all(&zip_data) {
            error!("Failed to write ZIP data to file: {:?}. Error: {:?}", zip_path, e);
            return Err(e.into());
        }
        info!("ZIP data successfully written to temporary file: {:?}", zip_path);

        // Extract the ZIP file
        let file = match File::open(&zip_path) {
            Ok(file) => {
                info!("Opened temporary ZIP file for extraction: {:?}", zip_path);
                file
            }
            Err(e) => {
                error!("Failed to open temporary ZIP file for extraction: {:?}. Error: {:?}", zip_path, e);
                return Err(e.into());
            }
        };

        let mut archive = match ZipArchive::new(file) {
            Ok(archive) => {
                info!("Successfully read ZIP archive: {:?}", zip_path);
                archive
            }
            Err(e) => {
                error!("Failed to read ZIP archive: {:?}. Error: {:?}", zip_path, e);
                return Err(e.into());
            }
        };

        let mut extracted_files = Vec::new();

        for i in 0..archive.len() {
            let mut file = match archive.by_index(i) {
                Ok(file) => file,
                Err(e) => {
                    warn!("Failed to read file at index {} in ZIP archive. Error: {:?}", i, e);
                    continue;
                }
            };

            let outpath = Path::new(output_dir).join(file.mangled_name());

            if file.is_dir() {
                if let Err(e) = create_dir_all(&outpath) {
                    warn!("Failed to create directory: {:?}. Error: {:?}", outpath, e);
                    continue;
                }
            } else {
                let mut outfile = match File::create(&outpath) {
                    Ok(outfile) => outfile,
                    Err(e) => {
                        warn!("Failed to create file: {:?}. Error: {:?}", outpath, e);
                        continue;
                    }
                };

                if let Err(e) = copy(&mut file, &mut outfile) {
                    warn!("Failed to extract file: {:?}. Error: {:?}", outpath, e);
                    continue;
                }

                extracted_files.push(outpath.to_string_lossy().to_string());
            }
        }

        // Remove the ZIP file
        if let Err(e) = fs::remove_file(&zip_path) {
            warn!("Failed to remove temporary ZIP file: {:?}. Error: {:?}", zip_path, e);
        } else {
            info!("Successfully removed temporary ZIP file: {:?}", zip_path);
        }

        info!("Completed extraction of ZIP file for S3 key: {}", s3_key);
        Ok(extracted_files)
    }

    /// Downloads and extracts a tar.gz file from S3.
    ///
    /// # Parameters
    /// - `s3_key`: The S3 key of the tar.gz file.
    /// - `output_dir`: The directory where the tar.gz file will be extracted.
    ///
    /// # Returns
    /// - `Ok(Vec<String>)`: A list of file paths extracted from the tar.gz file.
    /// - `Err(AppError)`: An error if the download or extraction fails.
    pub async fn download_and_extract_tar_gz(
        &self,
        s3_key: &str,
        output_dir: &str,
    ) -> Result<Vec<String>, AppError> {
        info!("Starting download and extraction of tar.gz file for S3 key: {}", s3_key);

        // Ensure the output directory exists
        if let Err(e) = create_dir_all(output_dir) {
            error!("Failed to create output directory: {:?}. Error: {:?}", output_dir, e);
            return Err(AppError::FileIoError(io::Error::new(io::ErrorKind::NotFound, "Failed to create output directory")));
        }

        // Download the tar.gz file from S3
        let tar_gz_data = match self.clients.get_s3_client().download_file(s3_key).await {
            Ok(data) => {
                info!("Successfully downloaded tar.gz file from S3: {}", s3_key);
                data
            }
            Err(e) => {
                error!("Failed to download tar.gz file from S3: {}. Error: {:?}", s3_key, e);
                return Err(e.into());
            }
        };

        // Save the tar.gz file locally
        let tar_gz_path = Path::new(output_dir).join("temp.tar.gz");
        let mut file = match File::create(&tar_gz_path) {
            Ok(file) => {
                info!("Temporary tar.gz file created at: {:?}", tar_gz_path);
                file
            }
            Err(e) => {
                error!("Failed to create temporary tar.gz file: {:?}. Error: {:?}", tar_gz_path, e);
                return Err(e.into());
            }
        };

        if let Err(e) = file.write_all(&tar_gz_data) {
            error!("Failed to write tar.gz data to file: {:?}. Error: {:?}", tar_gz_path, e);
            return Err(e.into());
        }
        info!("tar.gz data successfully written to temporary file: {:?}", tar_gz_path);

        // Extract the files directly into the output directory
        let status = match Command::new("tar")
            .arg("-xzf")
            .arg(&tar_gz_path)
            .arg("-C")
            .arg(output_dir)  // No "extracted_files" subdirectory
            .status()
        {
            Ok(status) => status,
            Err(e) => {
                error!("Failed to execute `tar` command for extraction. Error: {:?}", e);
                return Err(AppError::FileIoError(io::Error::new(io::ErrorKind::Other, "Failed to execute `tar` command")));
            }
        };

        if !status.success() {
            error!("Failed to extract tar.gz file. Command exited with status: {:?}", status);
            return Err(AppError::FileIoError(io::Error::new(io::ErrorKind::Other, "Failed to extract tar.gz")));
        }
        info!("Successfully extracted tar.gz file to: {:?}", output_dir);

        // Clean up the temporary tar.gz file
        if let Err(e) = fs::remove_file(&tar_gz_path) {
            warn!("Failed to remove temporary tar.gz file: {:?}. Error: {:?}", tar_gz_path, e);
        } else {
            info!("Successfully removed temporary tar.gz file: {:?}", tar_gz_path);
        }

        info!("Completed extraction of tar.gz file for S3 key: {}", s3_key);
        Ok(vec!["Extracted files successfully.".to_string()])
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
