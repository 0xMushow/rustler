use log::{error, info};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::clients::{
    s3_client::S3Client,
    postgres_client::PostgresClient,
    redis_client::RedisClient
};

/// A struct that holds all the clients required by the application.
///
/// # Fields
///
/// * `s3_client` - An instance of the S3 client.
/// * `postgres_client` - An instance of the PostgreSQL client.
/// * `redis_client` - An instance of the Redis client.
///
pub struct Clients {
    s3_client: S3Client,
    postgres_client: PostgresClient,
    redis_client: RedisClient,
}

/// Implementation block for `Clients`.
/// Contains methods to create and test connections to the clients.
///
/// # Methods
///
/// * `new` - Creates a new instance of `Clients`.
/// * `test_connections` - Tests the connections to all the clients.
///
impl Clients {
    /// Creates a new instance of `Clients`.
    pub async fn new(config: &AppConfig) -> Result<Self, AppError> {
        Ok(Self {
            s3_client: S3Client::new(config),
            postgres_client: PostgresClient::new(config).await?,
            redis_client: RedisClient::new(config)?,
        })
    }

    /// Tests the connections to all the clients.
    pub async fn test_connections(&self) -> Result<(), AppError> {
        if let Err(e) = self.s3_client.test_connection().await {
            error!("Failed to connect to S3: {}", e);
            return Err(e.into());
        }
        info!("S3 connection established successfully!");

        if let Err(e) = self.postgres_client.test_connection().await {
            error!("Failed to connect to PostgreSQL: {}", e);
            return Err(e.into());
        }
        info!("PostgreSQL connection established successfully!");

        if let Err(e) = self.redis_client.test_connection().await {
            error!("Failed to connect to Redis: {}", e);
            return Err(e.into());
        }
        info!("Redis connection established successfully!");

        Ok(())
    }

    /// Returns a reference to the S3 client.
    pub fn get_s3_client(&self) -> S3Client {
        self.s3_client.clone()
    }

    /// Returns a reference to the PostgreSQL client.
    pub fn get_postgres_client(&self) -> PostgresClient {
        self.postgres_client.clone()
    }

    /// Returns a reference to the Redis client.
    pub fn get_redis_client(&self) -> RedisClient {
        self.redis_client.clone()
    }
}
