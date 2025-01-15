use thiserror::Error;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::error::SdkError;
use sqlx::Error as SqlxError;
use redis::RedisError;

/// Represents custom errors that can occur in the application.
///
/// This enum encapsulates all possible errors that might arise during the execution
/// of the application, including environment variable errors, S3 connection errors,
/// PostgreSQL connection errors, and Redis connection errors.
#[derive(Error, Debug)]
pub enum AppError {
    /// An error indicating that a required environment variable is missing or invalid.
    #[error("Environment variable error: {0}")]
    EnvVarError(String),

    /// An error indicating a failure to connect to or interact with AWS S3.
    #[error("S3 connection error: {0}")]
    S3ConnectionError(#[from] SdkError<ListObjectsV2Error>),

    /// An error indicating a failure to connect to or interact with a PostgreSQL database.
    #[error("PostgreSQL connection error: {0}")]
    PostgresConnectionError(#[from] SqlxError),

    /// An error indicating a failure to connect to or interact with a Redis server.
    #[error("Redis connection error: {0}")]
    RedisConnectionError(#[from] RedisError),
}