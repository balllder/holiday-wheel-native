use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

/// Application error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Item not found")]
    NotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Error response returned to clients
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"error": "Item not found"}))]
pub struct ErrorResponse {
    /// Error message
    #[schema(example = "Item not found")]
    pub error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Item not found"),
            AppError::InvalidInput(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Convenience type for Result with AppError
pub type Result<T> = std::result::Result<T, AppError>;
