use crate::clients::clients::Clients;
use crate::error::AppError;
use axum::http::StatusCode;
use redis::{AsyncCommands, RedisError};
use axum::response::IntoResponse;

static CACHE_EXPIRATION: u64 = 60; // Cache expiration in seconds

/// Health check types for different services
#[derive(Clone)]
pub enum HealthCheckType {
    All,
    S3,
    Postgres,
    Redis,
}

impl HealthCheckType {
    /// Returns the success message for each health check type
    ///
    /// # Returns
    ///
    /// - `String`: The success message for the health check type.
    fn get_success_message(&self) -> String {
        match self {
            HealthCheckType::All => "All services are healthy",
            HealthCheckType::S3 => "S3 is healthy",
            HealthCheckType::Postgres => "PostgreSQL is healthy",
            HealthCheckType::Redis => "Redis is healthy",
        }.to_string()
    }

    /// Performs the actual health check for the services
    ///
    /// # Arguments
    ///
    /// - `clients`: A reference to the `Clients` struct.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the health check is successful.
    /// - `Err(String)`: If the health check fails.
    async fn check_health(&self, clients: &Clients) -> Result<(), String> {
        match self {
            HealthCheckType::All => {
                clients.get_s3_client().test_connection().await
                    .map_err(|e| format!("S3 Health Check Failed: {}", e))?;
                clients.get_postgres_client().test_connection().await
                    .map_err(|e| format!("PostgreSQL Health Check Failed: {}", e))?;
                clients.get_redis_client().test_connection().await
                    .map_err(|e| format!("Redis Health Check Failed: {}", e))?;
            },
            HealthCheckType::S3 => {
                clients.get_s3_client().test_connection().await
                    .map_err(|e| format!("S3 Health Check Failed: {}", e))?;
            },
            HealthCheckType::Postgres => {
                clients.get_postgres_client().test_connection().await
                    .map_err(|e| format!("PostgreSQL Health Check Failed: {}", e))?;
            },
            HealthCheckType::Redis => {
                clients.get_redis_client().test_connection().await
                    .map_err(|e| format!("Redis Health Check Failed: {}", e))?;
            },
        }
        Ok(())
    }
}

/// Perform the health check and cache the result if successful
///
/// # Arguments
///
/// - `clients`: A reference to the `Clients` struct.
/// - `check_type`: The type of health check to perform.
///
pub async fn perform_health_check(
    clients: &Clients,
    check_type: HealthCheckType,
) -> impl IntoResponse {
    // Try to return cached result first
    if let Ok(cached_result) = get_cached_health_check_status(clients).await {
        return cached_result;
    }

    // Perform the actual health check if cache miss
    let response = match check_type.check_health(clients).await {
        Ok(()) => {
            let response = (StatusCode::OK, check_type.get_success_message());

            // Cache the result after success
            if let Err(e) = cache_health_check_status(clients, &response).await {
                return (StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to cache health check status: {}", e));
            }

            response
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
    };

    response
}

/// Retrieve cached health check result from Redis
///
/// # Arguments
///
/// - `clients`: A reference to the `Clients` struct.
///
async fn get_cached_health_check_status(
    clients: &Clients,
) -> Result<(StatusCode, String), AppError> {
    let mut con = clients.get_redis_client()
        .get_client()
        .get_multiplexed_async_connection()
        .await?;

    let cached_result: Option<String> = con.get("health_check_status").await?;

    if let Some(cached) = cached_result {
        return Ok((StatusCode::OK, cached));
    }

    Err(AppError::RedisConnectionError(
        RedisError::from((redis::ErrorKind::TypeError, "Cache not found or expired"))
    ))
}

/// Cache the health check result in Redis
///
/// # Arguments
/// - `clients`: A reference to the `Clients` struct.
/// - `status`: A tuple containing the status code and message to cache.
///
async fn cache_health_check_status(
    clients: &Clients,
    status: &(StatusCode, String),
) -> Result<(), AppError> {
    let mut con = clients.get_redis_client()
        .get_client()
        .get_multiplexed_async_connection()
        .await?;

    let _: () = con.set_ex(
        "health_check_status",
        &status.1,
        CACHE_EXPIRATION
    ).await?;

    Ok(())
}
