use sqlx::PgPool;
use crate::config::AppConfig;
use crate::error::AppError;

/// A client for interacting with a PostgreSQL database.
///
/// This struct encapsulates a connection pool to a PostgreSQL database and provides
/// methods for testing the connection and performing database operations.
pub struct PostgresClient {
    pool: PgPool,
}

impl PostgresClient {
    /// Creates a new `PostgresClient` instance using the provided configuration.
    ///
    /// # Arguments
    /// - `config`: A reference to the `AppConfig` struct containing the database connection URL.
    ///
    /// # Returns
    /// - `Ok(Self)`: A new `PostgresClient` instance if the connection is successful.
    /// - `Err(AppError)`: An error if the connection to the database fails.
    pub async fn new(config: &AppConfig) -> Result<Self, AppError> {
        let pool = PgPool::connect(&config.database_url).await?;
        Ok(Self { pool })
    }

    /// Tests the connection to the PostgreSQL database.
    ///
    /// This method performs a simple query to verify that the database is reachable
    /// and responsive.
    ///
    /// # Returns
    /// - `Ok(())`: If the connection test is successful.
    /// - `Err(AppError)`: If the connection test fails.
    pub async fn test_connection(&self) -> Result<(), AppError> {
        let _: (i64,) = sqlx::query_as("SELECT $1")
            .bind(1_i64)
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}