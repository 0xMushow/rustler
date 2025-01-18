use axum::http::StatusCode;
use std::collections::HashMap;

/// A struct to represent a file type.
/// This struct contains information about the file type, such as the name,
/// allowed extensions, content types, magic numbers, and maximum file size.
///
/// # Fields
/// - `name`: The name of the file type.
/// - `extensions`: A list of allowed file extensions.
/// - `content_types`: A list of allowed content types.
/// - `magic_numbers`: A list of magic numbers to validate the file content.
/// - `max_size`: The maximum allowed file size in bytes.
///
#[derive(Debug, Clone)]
pub struct FileType {
    pub name: String,
    pub extensions: Vec<String>,
    pub content_types: Vec<String>,
    pub magic_numbers: Vec<Vec<u8>>,
    pub max_size: usize,
}

/// A struct to represent a file validation error.
/// This struct contains an HTTP status code and an error message.
/// The status code is used to set the HTTP status code in the response,
/// and the message is used to provide additional information about the error.
///
/// # Fields
/// - `code`: The HTTP status code.
/// - `message`: The error message.
///
pub struct FileValidationError {
    pub code: StatusCode,
    pub message: String,
}

impl FileType {
    /// Creates a new `FileType` instance with the provided parameters.
    ///
    /// # Parameters
    /// - `name`: The name of the file type.
    /// - `extensions`: A list of allowed file extensions.
    /// - `content_types`: A list of allowed content types.
    /// - `magic_numbers`: A list of magic numbers to validate the file content.
    /// - `max_size`: The maximum allowed file size in bytes.
    ///
    pub fn new(
        name: &str,
        extensions: Vec<&str>,
        content_types: Vec<&str>,
        magic_numbers: Vec<Vec<u8>>,
        max_size: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
            content_types: content_types.iter().map(|s| s.to_string()).collect(),
            magic_numbers,
            max_size,
        }
    }

    /// Validates the file extension.
    ///
    /// # Parameters
    /// - `filename`: The name of the file to validate.
    ///
    /// # Returns
    /// - `true` if the file extension is valid.
    /// - `false` if the file extension is invalid
    ///
    pub fn validate_extension(&self, filename: &str) -> bool {
        let extension = filename.split('.').last().unwrap_or("");
        self.extensions
            .iter()
            .any(|ext| ext.eq_ignore_ascii_case(extension))
    }

    /// Validates whether the provided content type matches one of the allowed types.
    ///
    /// # Parameters
    /// - `content_type`: The content type string to check (e.g., `application/zip`).
    ///
    /// # Returns
    /// - `true` if the content type is allowed.
    /// - `false` otherwise.
    ///
    /// # Example
    /// ```
    /// let file_type = FileType::new("ZIP", vec![], vec!["application/zip"], vec![], 100 * 1024 * 1024);
    /// assert!(file_type.validate_content_type("application/zip"));
    /// assert!(!file_type.validate_content_type("image/png"));
    /// ```
    ///
    pub fn validate_content_type(&self, content_type: &str) -> bool {
        self.content_types
            .iter()
            .any(|ct| ct.eq_ignore_ascii_case(content_type))
    }

    /// Validates whether the provided data contains one of the allowed magic numbers.
    /// The magic number is a sequence of bytes that uniquely identifies the file format.
    ///
    /// # Parameters
    /// - `data`: The byte array to check for the magic number.
    ///
    /// # Returns
    /// - `true` if the magic number is found.
    /// - `false` otherwise.
    ///
    pub fn validate_magic_number(&self, data: &[u8]) -> bool {
        if self.magic_numbers.is_empty() {
            return true;
        }

        self.magic_numbers.iter().any(|magic| {
            data.len() >= magic.len() && data.starts_with(magic)
        })
    }
}

/// A struct to validate files based on their type.
pub struct FileValidator {
    file_types: HashMap<String, FileType>,
}

impl FileValidator {
    /// Creates a new `FileValidator` instance with the default file types.
    pub fn new() -> Self {
        let mut validator = Self {
            file_types: HashMap::new(),
        };
        validator.register_default_types();
        validator
    }

    /// Registers the default file types (ZIP, PNG, JPEG).
    /// This method is called by `new` to initialize the validator with the default file types.
    fn register_default_types(&mut self) {
        // ZIP File Type
        self.register_file_type(FileType::new(
            "ZIP",
            vec!["zip"],
            vec!["application/zip"],
            vec![vec![0x50, 0x4B, 0x03, 0x04]], // ZIP magic number
            100 * 1024 * 1024, // 100MB
        ));

        // PNG File Type
        self.register_file_type(FileType::new(
            "PNG",
            vec!["png"],
            vec!["image/png"],
            vec![vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]], // PNG magic number
            10 * 1024 * 1024, // 10MB
        ));

        // JPEG File Type
        self.register_file_type(FileType::new(
            "JPEG",
            vec!["jpg", "jpeg"],
            vec!["image/jpeg"],
            vec![
                vec![0xFF, 0xD8, 0xFF, 0xE0],
                vec![0xFF, 0xD8, 0xFF, 0xE1],
            ], // JPEG magic numbers
            10 * 1024 * 1024, // 10MB
        ));
    }

    /// Registers a new file type with the validator.
    pub fn register_file_type(&mut self, file_type: FileType) {
        self.file_types.insert(file_type.name.clone(), file_type);
    }

    /// Validates a file based on its type.
    /// This method reads the file content, validates the extension, content type,
    /// magic number, and size of the file.
    ///
    /// # Parameters
    /// - `file_type_name`: The name of the file type to validate.
    /// - `field`: The `axum::extract::multipart::Field` containing the file data.
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)`: The file content as a byte array if the file is valid.
    /// - `Err(FileValidationError)`: An error if the file is invalid.
    ///
    pub async fn validate_file(
        &self,
        file_type_name: &str,
        field: &mut axum::extract::multipart::Field<'_>,
    ) -> Result<Vec<u8>, FileValidationError> {
        let file_type = self.file_types.get(file_type_name).ok_or_else(|| FileValidationError {
            code: StatusCode::BAD_REQUEST,
            message: format!("Unsupported file type: {}", file_type_name),
        })?;

        // Validate filename and extension
        let filename = field.file_name().ok_or_else(|| FileValidationError {
            code: StatusCode::BAD_REQUEST,
            message: "No filename provided".to_string(),
        })?;

        if !file_type.validate_extension(filename) {
            return Err(FileValidationError {
                code: StatusCode::UNSUPPORTED_MEDIA_TYPE,
                message: format!("Invalid file extension. Allowed extensions: {:?}", file_type.extensions),
            });
        }

        // Validate content type
        let content_type = field.content_type().unwrap_or("");
        if !file_type.validate_content_type(content_type) {
            return Err(FileValidationError {
                code: StatusCode::UNSUPPORTED_MEDIA_TYPE,
                message: format!("Invalid content type. Allowed types: {:?}", file_type.content_types),
            });
        }

        // Read and validate file content
        let mut buffer = Vec::new();
        let mut total_bytes = 0;

        while let Some(chunk) = field.chunk().await.transpose() {
            let chunk = chunk.map_err(|e| FileValidationError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Failed to read chunk: {}", e),
            })?;

            total_bytes += chunk.len();
            if total_bytes > file_type.max_size {
                return Err(FileValidationError {
                    code: StatusCode::PAYLOAD_TOO_LARGE,
                    message: format!("File exceeds maximum allowed size of {} bytes", file_type.max_size),
                });
            }

            buffer.extend_from_slice(&chunk);

            // Validate magic number on first chunk
            if buffer.len() == chunk.len() && !file_type.validate_magic_number(&buffer) {
                return Err(FileValidationError {
                    code: StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    message: format!("Invalid file format for {}", file_type.name),
                });
            }
        }

        Ok(buffer)
    }

    /// Finds a file type by its extension.
    pub fn find_file_type_by_extension(&self, extension: &str) -> Option<&FileType> {
        self.file_types
            .values()
            .find(|file_type| file_type.validate_extension(extension))
    }
}