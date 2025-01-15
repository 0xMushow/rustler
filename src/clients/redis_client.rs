use redis::{AsyncCommands, Client};
use crate::config::AppConfig;
use crate::error::AppError;

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub fn new(config: &AppConfig) -> Result<Self, AppError> {
        let client = Client::open(config.redis_url.clone())?;
        Ok(Self { client })
    }

    pub async fn test_connection(&self) -> Result<(), AppError> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let _: () = con.set("test_key", "test_value").await?;
        let _: String = con.get("test_key").await?;
        Ok(())
    }
}