use sqlx::PgPool;
use crate::clients::{redis_client::RedisClient, s3_client::S3Client};

/// Represents the application state.
///
/// This struct holds the database connection pool, Redis client, and S3 client.
/// It is used to share these resources across the application.
///
/// # Fields
///
/// * `db_pool` - A connection pool to the PostgreSQL database.
/// * `redis_client` - An instance of the Redis client.
/// * `s3_client` - An instance of the S3 client.
///
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: RedisClient,
    pub s3_client: S3Client,
}

/// Implementation block for `AppState`.
impl AppState {
    /// Creates a new instance of `AppState`.
    ///
    /// # Arguments
    ///
    /// * `db_pool` - A connection pool to the PostgreSQL database.
    /// * `redis_client` - An instance of the Redis client.
    /// * `s3_client` - An instance of the S3 client.
    ///
    /// # Returns
    ///
    /// A new `AppState` instance with the provided resources.
    ///
    pub fn new(db_pool: PgPool, redis_client: RedisClient, s3_client: S3Client) -> Self {
        Self {
            db_pool,
            redis_client,
            s3_client,
        }
    }
}