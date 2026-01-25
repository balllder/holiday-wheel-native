//! Structured logging with request ID tracking
//!
//! Provides middleware for generating unique request IDs and adding them to
//! response headers and log spans.

use axum::{
    body::Body,
    extract::Request,
    http::{header::HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::{info_span, Instrument, Span};
use uuid::Uuid;

/// Header name for request ID
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Extension type for storing request ID in request extensions
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RequestId(pub String);

/// Middleware that generates a unique request ID for each request.
///
/// - Generates a UUID v4 for each incoming request
/// - Adds the request ID to the response headers as `X-Request-ID`
/// - Stores the request ID in request extensions for use in handlers
/// - Creates a tracing span with the request ID for structured logging
pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    // Generate unique request ID
    let request_id = Uuid::new_v4().to_string();

    // Store in request extensions for handler access
    request.extensions_mut().insert(RequestId(request_id.clone()));

    // Extract request details for logging
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    // Create span with request context
    let span = info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        uri = %uri,
        version = ?version,
    );

    // Execute request within the span
    let response = async move {
        tracing::info!("started processing request");
        let response = next.run(request).await;
        tracing::info!(status = %response.status(), "finished processing request");
        response
    }
    .instrument(span)
    .await;

    // Add request ID to response headers
    let (mut parts, body) = response.into_parts();
    if let Ok(header_name) = HeaderName::try_from(REQUEST_ID_HEADER) {
        if let Ok(header_value) = HeaderValue::from_str(&request_id) {
            parts.headers.insert(header_name, header_value);
        }
    }

    Response::from_parts(parts, body)
}

/// Get the current request ID from the tracing span, if available
#[allow(dead_code)]
pub fn current_request_id() -> Option<String> {
    Span::current()
        .field("request_id")
        .map(|f| f.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_header_name() {
        assert_eq!(REQUEST_ID_HEADER, "x-request-id");
    }

    #[test]
    fn test_request_id_clone() {
        let id = RequestId("test-123".to_string());
        let cloned = id.clone();
        assert_eq!(id.0, cloned.0);
    }
}
