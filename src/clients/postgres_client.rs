use sqlx::PgPool;
use crate::config::AppConfig;
use crate::error::AppError;

pub struct PostgresClient {
    pool: PgPool,
}

impl PostgresClient {
    pub async fn new(config: &AppConfig) -> Result<Self, AppError> {
        let pool = PgPool::connect(&config.database_url).await?;
        Ok(Self { pool })
    }

    pub async fn test_connection(&self) -> Result<(), AppError> {
        let _: (i64,) = sqlx::query_as("SELECT $1")
            .bind(1_i64)
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}