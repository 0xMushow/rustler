use aws_sdk_s3::{Client, config::{Credentials, Region}};
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
}