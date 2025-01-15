use log::{error, info};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::clients::{
    s3_client::S3Client,
    postgres_client::PostgresClient,
    redis_client::RedisClient
};

pub struct Clients {
    s3_client: S3Client,
    postgres_client: PostgresClient,
    redis_client: RedisClient,
}

impl Clients {
    pub async fn new(config: &AppConfig) -> Result<Self, AppError> {
        Ok(Self {
            s3_client: S3Client::new(config),
            postgres_client: PostgresClient::new(config).await?,
            redis_client: RedisClient::new(config)?,
        })
    }

    pub async fn test_connections(&self) -> Result<(), AppError> {
        if let Err(e) = self.s3_client.test_connection().await {
            error!("Failed to connect to S3: {}", e);
            return Err(e.into());
        } else {
            info!("S3 connection established successfully!");
        }

        if let Err(e) = self.postgres_client.test_connection().await {
            error!("Failed to connect to PostgreSQL: {}", e);
            return Err(e.into());
        } else {
            info!("PostgreSQL connection established successfully!");
        }

        if let Err(e) = self.redis_client.test_connection().await {
            error!("Failed to connect to Redis: {}", e);
            return Err(e.into());
        } else {
            info!("Redis connection established successfully!");
        }

        Ok(())
    }
}