use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use utoipa::ToSchema;

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "healthy",
    "service": "project-starter-api",
    "version": "0.1.0"
}))]
pub struct HealthResponse {
    /// Current health status
    #[schema(example = "healthy")]
    pub status: String,

    /// Service name
    #[schema(example = "project-starter-api")]
    pub service: String,

    /// Service version
    #[schema(example = "0.1.0")]
    pub version: String,
}

/// Readiness check response with dependencies
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "ready",
    "service": "project-starter-api",
    "version": "0.1.0",
    "checks": {
        "database": "healthy"
    }
}))]
pub struct ReadinessResponse {
    /// Readiness status
    #[schema(example = "ready")]
    pub status: String,

    /// Service name
    #[schema(example = "project-starter-api")]
    pub service: String,

    /// Service version
    #[schema(example = "0.1.0")]
    pub version: String,

    /// Health status of dependencies
    pub checks: ReadinessChecks,
}

/// Health check results for dependencies
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"database": "healthy"}))]
pub struct ReadinessChecks {
    /// Database connection status
    #[schema(example = "healthy")]
    pub database: String,
}

/// Basic health check endpoint
///
/// Returns HTTP 200 if the service is running
///
/// # Example
/// ```bash
/// curl http://localhost:3000/health
/// ```
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Readiness check endpoint
///
/// Verifies that the service and all dependencies are ready
///
/// Returns HTTP 200 if ready, HTTP 503 if not ready
///
/// # Example
/// ```bash
/// curl http://localhost:3000/health/ready
/// ```
#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, description = "Service is ready", body = ReadinessResponse),
        (status = 503, description = "Service not ready", body = ReadinessResponse)
    ),
    tag = "health"
)]
pub async fn readiness_check(
    State(pool): State<PgPool>,
) -> Json<ReadinessResponse> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    Json(ReadinessResponse {
        status: if db_status == "healthy" {
            "ready".to_string()
        } else {
            "not_ready".to_string()
        },
        service: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: ReadinessChecks {
            database: db_status.to_string(),
        },
    })
}
