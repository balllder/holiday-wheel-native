//! Project Starter API
//!
//! A template REST API with OpenAPI documentation

use utoipa::OpenApi;

// Re-export modules for use in binaries
pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod models;

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Project Starter API",
        version = "0.1.0",
        description = "A template REST API built with Rust, Axum, and PostgreSQL",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api.example.com", description = "Production server")
    ),
    paths(
        // Health endpoints
        api::health::health_check,
        api::health::readiness_check,
        // Items endpoints
        api::items::list_items,
        api::items::get_item,
        api::items::create_item,
    ),
    components(
        schemas(
            // Models
            models::Item,
            models::CreateItemRequest,
            // Health
            api::health::HealthResponse,
            api::health::ReadinessResponse,
            api::health::ReadinessChecks,
            // Errors
            error::ErrorResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "items", description = "Item management endpoints")
    )
)]
pub struct ApiDoc;
