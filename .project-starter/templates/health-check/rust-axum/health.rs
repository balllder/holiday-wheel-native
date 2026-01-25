// Health Check Endpoints
//
// Provides liveness and readiness endpoints for monitoring and orchestration.
//
// Usage:
// ```rust
// use crate::health::{health_check, ready_check};
//
// let app = Router::new()
//     .route("/health", get(health_check))
//     .route("/health/ready", get(ready_check));
// ```

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

/// Health check response structure
#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status: "healthy" or "unhealthy"
    pub status: String,

    /// Service name from environment or Cargo.toml
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// Service version from Cargo.toml
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Detailed health checks for dependencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HealthChecks>,
}

/// Detailed health checks for dependencies
#[derive(Serialize, Deserialize)]
pub struct HealthChecks {
    /// Database health status
    pub database: String,

    /// Cache (Redis) health status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<String>,
}

/// Application state for health checks
pub struct AppState {
    pub db_pool: PgPool,
    // pub redis_client: redis::Client, // Uncomment if using Redis
}

/// Liveness probe endpoint
///
/// Returns 200 if the service is running.
/// Use this for Kubernetes liveness probes.
///
/// GET /health
/// Response: {"status": "healthy", "service": "my-service", "version": "0.1.0"}
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: Some(env!("CARGO_PKG_NAME").to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        checks: None,
    })
}

/// Readiness probe endpoint
///
/// Returns 200 if the service is ready to accept traffic.
/// Checks database and cache connections.
/// Use this for Kubernetes readiness probes and load balancer health checks.
///
/// GET /health/ready
/// Response: {"status": "ready", "checks": {"database": "ok", "cache": "ok"}}
pub async fn ready_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1")
        .execute(&state.db_pool)
        .await
    {
        Ok(_) => "ok",
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unhealthy".to_string(),
                    service: Some(env!("CARGO_PKG_NAME").to_string()),
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    checks: Some(HealthChecks {
                        database: "error".to_string(),
                        cache: None,
                    }),
                }),
            );
        }
    };

    // Optional: Check Redis connection
    // Uncomment if using Redis
    /*
    let cache_status = match state.redis_client.get_connection() {
        Ok(mut conn) => match redis::cmd("PING").query::<String>(&mut conn) {
            Ok(_) => Some("ok".to_string()),
            Err(e) => {
                tracing::error!("Redis health check failed: {}", e);
                Some("error".to_string())
            }
        },
        Err(e) => {
            tracing::error!("Redis connection failed: {}", e);
            Some("error".to_string())
        }
    };
    */

    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ready".to_string(),
            service: Some(env!("CARGO_PKG_NAME").to_string()),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            checks: Some(HealthChecks {
                database: db_status.to_string(),
                cache: None, // Replace with cache_status if using Redis
            }),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_healthy() {
        let response = health_check().await;
        assert_eq!(response.0.status, "healthy");
        assert_eq!(response.0.service, Some(env!("CARGO_PKG_NAME").to_string()));
        assert_eq!(response.0.version, Some(env!("CARGO_PKG_VERSION").to_string()));
    }
}
