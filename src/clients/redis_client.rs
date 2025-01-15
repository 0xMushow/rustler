use redis::{AsyncCommands, Client};
use crate::config::AppConfig;
use crate::error::AppError;

/// A client for interacting with a Redis server.
///
/// This struct encapsulates the connection to a Redis server and provides methods
/// for testing the connection and performing Redis operations.
pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    /// Creates a new `RedisClient` instance using the provided configuration.
    ///
    /// # Arguments
    /// - `config`: A reference to the `AppConfig` struct containing the Redis connection URL.
    ///
    /// # Returns
    /// - `Ok(Self)`: A new `RedisClient` instance if the connection is successful.
    /// - `Err(AppError)`: An error if the connection to Redis fails.
    pub fn new(config: &AppConfig) -> Result<Self, AppError> {
        let client = Client::open(config.redis_url.clone())?;
        Ok(Self { client })
    }

    /// Tests the connection to the Redis server.
    ///
    /// This method performs a simple set/get operation to verify that the Redis server
    /// is reachable and responsive.
    ///
    /// # Returns
    /// - `Ok(())`: If the connection test is successful.
    /// - `Err(AppError)`: If the connection test fails.
    pub async fn test_connection(&self) -> Result<(), AppError> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let _: () = con.set("test_key", "test_value").await?;
        let _: String = con.get("test_key").await?;
        Ok(())
    }
}