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
use redis::{AsyncCommands};
use zip::ZipArchive;
use crate::clients::clients::Clients;
use crate::error::AppError;
use crate::utils::file_utils::FileValidator;

/// Supported archive file types
#[derive(Debug)]
enum ArchiveType {
    Zip,
    TarGz,
}

impl ArchiveType {
    fn extension(&self) -> &str {
        match self {
            ArchiveType::Zip => ".zip",
            ArchiveType::TarGz => ".tar.gz",
        }
    }
}

/// A service to handle file-related operations.
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

    /// Detects the type of archive file based on the base name.
    ///
    /// This function checks if the archive file exists in S3 with the given base name
    ///
    /// # Parameters
    /// - `base_name`: The base name of the archive file
    async fn detect_archive_type(&self, base_name: &str) -> Result<(String, ArchiveType), AppError> {
        for archive_type in [ArchiveType::Zip, ArchiveType::TarGz] {
            let key = format!("{}{}", base_name, archive_type.extension());
            if self.clients.get_s3_client().file_exists(&key).await {
                return Ok((key, archive_type));
            }
        }

        error!("No archive file found for base name: {}", base_name);
        Err(
            AppError::ValidationError(format!(
                "No archive file found for base name: {}",
                base_name
            )))
    }

    /// Downloads and extracts an archive file from S3, automatically detecting the type
    ///
    /// # Parameters
    /// - `base_name`: The base name of the archive file
    /// - `output_dir`: The directory where the file will be extracted
    pub async fn download_and_extract_archive(
        &self,
        base_name: &str,
        output_dir: &str,
    ) -> Result<Vec<String>, AppError> {
        info!("Attempting to detect and extract archive for: {}", base_name);

        let (s3_key, archive_type) = self.detect_archive_type(base_name).await?;

        info!("Detected archive type {:?} at key: {}", archive_type, s3_key);

        match archive_type {
            ArchiveType::Zip => self.download_and_extract_zip(&s3_key, output_dir).await,
            ArchiveType::TarGz => self.download_and_extract_tar_gz(&s3_key, output_dir).await,
        }
    }

    /// Generic function to handle file download and temporary storage
    async fn download_to_temp_file(
        &self,
        s3_key: &str,
        output_dir: &str,
        temp_filename: &str,
    ) -> Result<std::path::PathBuf, AppError> {
        create_dir_all(output_dir).map_err(|e| {
            error!("Failed to create output directory: {}. Error: {:?}", output_dir, e);
            AppError::FileIoError(e)
        })?;

        let file_data = self.clients.get_s3_client().download_file(s3_key).await?;

        let temp_path = Path::new(output_dir).join(temp_filename);
        let mut file = File::create(&temp_path).map_err(|e| {
            error!("Failed to create temporary file: {:?}. Error: {:?}", temp_path, e);
            AppError::FileIoError(e)
        })?;

        file.write_all(&file_data).map_err(|e| {
            error!("Failed to write data to temporary file: {:?}. Error: {:?}", temp_path, e);
            AppError::FileIoError(e)
        })?;

        Ok(temp_path)
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
                        info!("Successfully uploaded file to S3: '{}'. Size: {} bytes", file_name, buffer.len());
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
                warn!("File validation failed for '{}': {}", file_name, validation_error.message);
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
    async fn download_and_extract_zip(
        &self,
        s3_key: &str,
        output_dir: &str,
    ) -> Result<Vec<String>, AppError> {
        info!("Starting download and extraction of ZIP file: {}", s3_key);

        let zip_path = self.download_to_temp_file(s3_key, output_dir, "temp.zip").await?;
        let mut extracted_files = Vec::new();

        let file = File::open(&zip_path).map_err(|e| {
            error!("Failed to open ZIP file for extraction: {:?}. Error: {:?}", zip_path, e);
            AppError::FileIoError(e)
        })?;

        let mut archive = ZipArchive::new(file).map_err(|e| {
            error!("Failed to read ZIP archive: {:?}. Error: {:?}", zip_path, e);
            AppError::FileIoError(io::Error::new(io::ErrorKind::Other, e))
        })?;

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
                if let Ok(mut outfile) = File::create(&outpath) {
                    if copy(&mut file, &mut outfile).is_ok() {
                        extracted_files.push(outpath.to_string_lossy().to_string());
                    }
                }
            }
        }

        if let Err(e) = fs::remove_file(&zip_path) {
            warn!("Failed to remove temporary ZIP file: {:?}. Error: {:?}", zip_path, e);
        }

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
    async fn download_and_extract_tar_gz(
        &self,
        s3_key: &str,
        output_dir: &str,
    ) -> Result<Vec<String>, AppError> {
        info!("Starting download and extraction of tar.gz file: {}", s3_key);

        let tar_gz_path = self.download_to_temp_file(s3_key, output_dir, "temp.tar.gz").await?;

        let status = Command::new("tar")
            .arg("-xzf")
            .arg(&tar_gz_path)
            .arg("-C")
            .arg(output_dir)
            .status()
            .map_err(|e| {
                error!("Failed to execute tar command. Error: {:?}", e);
                AppError::FileIoError(e)
            })?;

        if !status.success() {
            error!("tar command failed with status: {:?}", status);
            return Err(AppError::FileIoError(io::Error::new(
                io::ErrorKind::Other,
                "Failed to extract tar.gz",
            )));
        }

        if let Err(e) = fs::remove_file(&tar_gz_path) {
            warn!("Failed to remove temporary tar.gz file: {:?}. Error: {:?}", tar_gz_path, e);
        }

        Ok(vec!["Extraction completed successfully".to_string()])
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

    pub async fn get_cached_file(&self, base_name: &str) -> Result<Option<String>, AppError> {
        let mut con = self.clients
            .get_redis_client()
            .get_client()
            .get_multiplexed_async_connection()
            .await
            .map_err(AppError::RedisConnectionError)?;

        let cached_file: Option<String> = con
            .get(format!("file_cache:{}", base_name))
            .await
            .map_err(AppError::RedisConnectionError)?;

        Ok(cached_file)
    }

    /// Caches the extracted files in Redis after downloading and extracting.
    pub async fn cache_files(&self, base_name: &str, files: &[String]) -> Result<(), AppError> {
        let mut con = self.clients
            .get_redis_client()
            .get_client()
            .get_multiplexed_async_connection()
            .await
            .map_err(AppError::RedisConnectionError)?;

        let cache_key = format!("file_cache:{}", base_name);
        let files_json = serde_json::to_string(&files)
            .map_err(AppError::SerializationError)?;

        let _: () = con.set_ex(cache_key, files_json, 3600)
            .await
            .map_err(AppError::RedisConnectionError)?;

        Ok(())
    }
}