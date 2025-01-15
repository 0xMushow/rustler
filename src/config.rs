use std::env;
use crate::error::AppError;

#[derive(Clone)]
pub struct AppConfig {
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_region: String,
    pub s3_bucket_name: String,
    pub database_url: String,
    pub redis_url: String,
}

fn get_env_var(key: &str) -> Result<String, AppError> {
    env::var(key).map_err(|_| AppError::EnvVarError(format!("{} not set", key)))
}

impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
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