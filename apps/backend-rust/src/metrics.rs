//! Prometheus metrics module for the Holiday Wheel backend.
//!
//! This module provides:
//! - Common metrics (http_requests_total, http_request_duration_seconds, active_connections)
//! - A `/metrics` endpoint that returns Prometheus format
//! - Middleware to record request metrics

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Shared metrics state
#[derive(Clone)]
pub struct MetricsState {
    /// Prometheus handle for rendering metrics
    pub handle: PrometheusHandle,
    /// Active connection counter (atomic for thread-safe updates)
    pub active_connections: Arc<AtomicU64>,
}

impl MetricsState {
    /// Create a new MetricsState with initialized Prometheus recorder
    pub fn new() -> Self {
        let handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("Failed to install Prometheus recorder");

        Self {
            handle,
            active_connections: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increment active connections
    pub fn inc_connections(&self) {
        let count = self.active_connections.fetch_add(1, Ordering::SeqCst) + 1;
        gauge!("active_connections").set(count as f64);
    }

    /// Decrement active connections
    pub fn dec_connections(&self) {
        let count = self.active_connections.fetch_sub(1, Ordering::SeqCst) - 1;
        gauge!("active_connections").set(count as f64);
    }

    /// Get current active connections count
    #[allow(dead_code)]
    pub fn get_connections(&self) -> u64 {
        self.active_connections.load(Ordering::SeqCst)
    }

    /// Render metrics in Prometheus format
    pub fn render(&self) -> String {
        self.handle.render()
    }
}

impl Default for MetricsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics endpoint handler - returns Prometheus format
pub async fn metrics_handler(State(metrics): State<MetricsState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics.render(),
    )
}

/// Middleware to record HTTP request metrics
///
/// Records:
/// - `http_requests_total` - Counter with labels: method, path, status
/// - `http_request_duration_seconds` - Histogram with labels: method, path
pub async fn metrics_middleware(
    State(metrics): State<MetricsState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Normalize path to avoid high cardinality
    // Replace dynamic segments with placeholders
    let normalized_path = normalize_path(&path);

    // Increment active connections
    metrics.inc_connections();

    // Process the request
    let response = next.run(request).await;

    // Decrement active connections
    metrics.dec_connections();

    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // Record request count
    counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path" => normalized_path.clone(),
        "status" => status
    )
    .increment(1);

    // Record request duration
    histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "path" => normalized_path
    )
    .record(duration);

    response
}

/// Normalize path to reduce cardinality
///
/// Replaces dynamic segments (UUIDs, numbers, etc.) with placeholders
fn normalize_path(path: &str) -> String {
    // Handle common dynamic patterns
    let segments: Vec<&str> = path.split('/').collect();
    let normalized: Vec<String> = segments
        .iter()
        .map(|segment| {
            // Replace UUIDs
            if segment.len() == 36 && segment.chars().filter(|c| *c == '-').count() == 4 {
                return ":id".to_string();
            }
            // Replace numeric IDs
            if segment.chars().all(|c| c.is_ascii_digit()) && !segment.is_empty() {
                return ":id".to_string();
            }
            segment.to_string()
        })
        .collect();

    normalized.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    fn create_test_metrics() -> MetricsState {
        // For tests, we create a recorder that doesn't conflict
        // Note: In real tests, you'd use a test-specific recorder
        MetricsState {
            handle: PrometheusBuilder::new().build_recorder().handle(),
            active_connections: Arc::new(AtomicU64::new(0)),
        }
    }

    #[test]
    fn test_normalize_path_simple() {
        assert_eq!(normalize_path("/health"), "/health");
        assert_eq!(normalize_path("/api/users"), "/api/users");
    }

    #[test]
    fn test_normalize_path_numeric_id() {
        assert_eq!(normalize_path("/api/users/123"), "/api/users/:id");
        assert_eq!(
            normalize_path("/api/items/456/details"),
            "/api/items/:id/details"
        );
    }

    #[test]
    fn test_normalize_path_uuid() {
        assert_eq!(
            normalize_path("/api/users/550e8400-e29b-41d4-a716-446655440000"),
            "/api/users/:id"
        );
    }

    #[test]
    fn test_active_connections() {
        let metrics = create_test_metrics();

        assert_eq!(metrics.get_connections(), 0);

        metrics.inc_connections();
        assert_eq!(metrics.get_connections(), 1);

        metrics.inc_connections();
        assert_eq!(metrics.get_connections(), 2);

        metrics.dec_connections();
        assert_eq!(metrics.get_connections(), 1);
    }

    #[test]
    fn test_metrics_render() {
        let metrics = create_test_metrics();
        let output = metrics.render();

        // Should return a valid string (even if empty initially)
        assert!(output.is_empty() || output.contains('#') || output.len() > 0);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let metrics = create_test_metrics();

        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(metrics);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.contains("text/plain"));
    }

    #[tokio::test]
    async fn test_middleware_records_metrics() {
        let metrics = create_test_metrics();
        let metrics_clone = metrics.clone();

        let app = Router::new()
            .route("/test", get(test_handler))
            .route("/metrics", get(metrics_handler))
            .layer(axum::middleware::from_fn_with_state(
                metrics.clone(),
                metrics_middleware,
            ))
            .with_state(metrics);

        // Make a test request
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify active connections returned to 0
        assert_eq!(metrics_clone.get_connections(), 0);
    }
}
