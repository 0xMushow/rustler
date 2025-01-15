use thiserror::Error;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::error::SdkError;
use sqlx::Error as SqlxError;
use redis::RedisError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Environment variable error: {0}")]
    EnvVarError(String),

    #[error("S3 connection error: {0}")]
    S3ConnectionError(#[from] SdkError<ListObjectsV2Error>),

    #[error("PostgreSQL connection error: {0}")]
    PostgresConnectionError(#[from] SqlxError),

    #[error("Redis connection error: {0}")]
    RedisConnectionError(#[from] RedisError),
}