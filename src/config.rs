use std::env;
use crate::error::AppError;

/// Represents the application configuration loaded from environment variables.
///
/// This struct holds all the necessary configuration values required to connect
/// to external services like AWS S3, PostgreSQL (RDS), and Redis.
#[derive(Clone)]
pub struct AppConfig {
    /// AWS access key ID for authenticating with AWS services.
    pub aws_access_key_id: String,

    /// AWS secret access key for authenticating with AWS services.
    pub aws_secret_access_key: String,

    /// AWS region where the S3 bucket is located.
    pub aws_region: String,

    /// Name of the S3 bucket used for file storage.
    pub s3_bucket_name: String,

    /// Connection URL for the PostgreSQL database (RDS).
    pub database_url: String,

    /// Connection URL for the Redis server.
    pub redis_url: String,
}

/// Fetches an environment variable by its key.
///
/// # Arguments
/// - `key`: The name of the environment variable to fetch.
///
/// # Returns
/// - `Ok(String)`: The value of the environment variable if it exists.
/// - `Err(AppError)`: An error if the environment variable is not set.
fn get_env_var(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::EnvVarError(format!("{} not set", key)))
}

impl AppConfig {
    /// Loads the application configuration from environment variables.
    ///
    /// This function reads the `.env` file (if present) and fetches all required
    /// environment variables. If any required variable is missing, an error is returned.
    ///
    /// # Returns
    /// - `Ok(Self)`: The loaded configuration if all environment variables are set.
    /// - `Err(AppError)`: An error if any required environment variable is missing.
    pub fn from_env() -> Result<Self, AppError> {
        // Load the `.env` file if it exists.
        dotenv::dotenv().ok();

        Ok(Self {
            aws_access_key_id: get_env_var("AWS_ACCESS_KEY_ID")?,
            aws_secret_access_key: get_env_var("AWS_SECRET_ACCESS_KEY")?,
            aws_region: get_env_var("AWS_REGION")?,
            s3_bucket_name: get_env_var("S3_BUCKET_NAME")?,
            database_url: get_env_var("DATABASE_URL")?,
            redis_url: get_env_var("REDIS_URL")?,
        })
    }
}