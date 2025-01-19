use std::error::Error;
use aws_sdk_s3::{Client, config::{Credentials, Region}};
use aws_sdk_s3::primitives::ByteStream;
use crate::config::AppConfig;
use crate::error::AppError;

/// Client for interacting with AWS S3.
#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket_name: String,
}

impl S3Client {
    /// Creates a new S3 client using the provided configuration.
    pub fn new(config: &AppConfig) -> Self {
        let credentials = Credentials::new(
            config.aws_access_key_id.clone(),
            config.aws_secret_access_key.clone(),
            None,
            None,
            "loaded-from-env",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .region(Region::new(config.aws_region.clone()))
            .credentials_provider(credentials)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket_name: config.s3_bucket_name.clone(),
        }
    }

    /// Tests the connection to the S3 bucket by listing objects.
    ///
    /// # Returns
    /// - `Ok(())` if the connection is successful.
    /// - `Err(AppError)` if the connection fails.
    pub async fn test_connection(&self) -> Result<(), AppError> {
        self.client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .send()
            .await?;
        Ok(())
    }

    /// Returns a reference to the S3 client.
    ///
    /// # Returns
    /// - A reference to the S3 client.
    ///
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    /// Returns the name of the S3 bucket.
    ///
    /// # Returns
    /// - The name of the S3 bucket.
    ///
    pub fn get_bucket_name(&self) -> String {
        self.bucket_name.clone()
    }

    /// Uploads a file to the S3 bucket.
    ///
    /// # Parameters
    /// - `file_name` - The name of the file to upload.
    /// - `data` - The file content as a byte array.
    ///
    pub async fn upload_file(&self, file_name: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let byte_stream = ByteStream::from(data.to_vec());
        self.get_client()
            .put_object()
            .bucket(self.get_bucket_name())
            .key(file_name)
            .body(byte_stream)
            .send()
            .await?;
        Ok(())
    }

    /// Downloads a file from the S3 bucket.
    ///
    /// # Parameters
    /// - `key` - The key of the file to download.
    ///
    pub async fn download_file(&self, key: &str) -> Result<Vec<u8>, AppError> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await?;

        let data = response.body.collect().await?;
        Ok(data.into_bytes().to_vec())
    }

    /// Checks if a file exists in the S3 bucket.
    ///
    /// # Parameters
    /// - `key` - The key of the file to check.
    ///
    pub async fn file_exists(&self, key: &str) -> bool {
        match self.get_client().head_object().bucket(&self.get_bucket_name()).key(key).send().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}