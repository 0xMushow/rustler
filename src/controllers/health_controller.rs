use axum::{Extension, response::IntoResponse};
use crate::services::health_service::{perform_health_check, HealthCheckType};
use std::sync::Arc;
use crate::clients::clients::Clients;

/// Handler for checking all services
pub async fn health_check_handler(Extension(state): Extension<Arc<Clients>>) -> impl IntoResponse {
    perform_health_check(state.as_ref(), HealthCheckType::All).await
}

/// Handler for checking S3 health
pub async fn s3_health_check_handler(Extension(state): Extension<Arc<Clients>>) -> impl IntoResponse {
    perform_health_check(state.as_ref(), HealthCheckType::S3).await
}

/// Handler for checking PostgreSQL health
pub async fn postgres_health_check_handler(Extension(state): Extension<Arc<Clients>>) -> impl IntoResponse {
    perform_health_check(state.as_ref(), HealthCheckType::Postgres).await
}

/// Handler for checking Redis health
pub async fn redis_health_check_handler(Extension(state): Extension<Arc<Clients>>) -> impl IntoResponse {
    perform_health_check(state.as_ref(), HealthCheckType::Redis).await
}
