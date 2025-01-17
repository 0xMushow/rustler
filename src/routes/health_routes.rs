use axum::{Router, routing::get};
use crate::controllers::health_controller::{health_check_handler, s3_health_check_handler, postgres_health_check_handler, redis_health_check_handler};

/// Returns a router with all health check endpoints
///
/// # Returns
/// A Router containing the following endpoints:
/// - GET /health - Checks all services
/// - GET /health/s3 - Checks S3 only
/// - GET /health/postgres - Checks PostgreSQL only
/// - GET /health/redis - Checks Redis only
///
pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/health/s3", get(s3_health_check_handler))
        .route("/health/postgres", get(postgres_health_check_handler))
        .route("/health/redis", get(redis_health_check_handler))
}
