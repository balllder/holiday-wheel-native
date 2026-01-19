//! Security headers middleware for the Holiday Wheel backend.
//!
//! Adds essential security headers to all HTTP responses to protect against
//! common web vulnerabilities like XSS, clickjacking, and MIME-sniffing attacks.

use axum::http::{header, Request, Response};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Security headers values
const X_CONTENT_TYPE_OPTIONS: &str = "nosniff";
const X_FRAME_OPTIONS: &str = "DENY";
const X_XSS_PROTECTION: &str = "1; mode=block";
const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";

/// Content Security Policy
/// - default-src 'self': Only allow resources from same origin by default
/// - script-src 'self' 'unsafe-inline' + CDNs: Allow scripts from same origin, inline, Socket.IO and jsdelivr (QR code lib)
/// - style-src 'self' 'unsafe-inline' + Google Fonts: Allow styles from same origin, inline, and Google Fonts
/// - font-src 'self' + Google Fonts: Allow fonts from same origin and Google Fonts
/// - img-src 'self' data: blob:: Allow images from same origin, data URIs, and blob URLs
/// - connect-src 'self' wss: ws:: Allow connections to same origin and WebSockets
/// - frame-ancestors 'none': Prevent embedding in frames (reinforces X-Frame-Options)
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.socket.io https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: blob:; connect-src 'self' wss: ws:; frame-ancestors 'none'";

/// Layer that adds security headers to all responses.
#[derive(Clone, Debug)]
pub struct SecurityHeadersLayer;

impl SecurityHeadersLayer {
    /// Creates a new `SecurityHeadersLayer`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SecurityHeadersLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersService { inner }
    }
}

/// Service that wraps an inner service and adds security headers to responses.
#[derive(Clone, Debug)]
pub struct SecurityHeadersService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SecurityHeadersService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = SecurityHeadersFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        SecurityHeadersFuture {
            inner: self.inner.call(request),
        }
    }
}

/// Future that adds security headers to the response.
#[pin_project::pin_project]
pub struct SecurityHeadersFuture<F> {
    #[pin]
    inner: F,
}

impl<F, ResBody, E> std::future::Future for SecurityHeadersFuture<F>
where
    F: std::future::Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => Poll::Ready(result.map(|mut response| {
                let headers = response.headers_mut();

                // Add security headers
                headers.insert(
                    header::X_CONTENT_TYPE_OPTIONS,
                    X_CONTENT_TYPE_OPTIONS.parse().unwrap(),
                );
                headers.insert(
                    header::X_FRAME_OPTIONS,
                    X_FRAME_OPTIONS.parse().unwrap(),
                );
                headers.insert(
                    header::X_XSS_PROTECTION,
                    X_XSS_PROTECTION.parse().unwrap(),
                );
                headers.insert(
                    header::REFERRER_POLICY,
                    REFERRER_POLICY.parse().unwrap(),
                );
                headers.insert(
                    header::CONTENT_SECURITY_POLICY,
                    CONTENT_SECURITY_POLICY.parse().unwrap(),
                );

                response
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn hello_handler() -> &'static str {
        "Hello, World!"
    }

    fn create_test_app() -> Router {
        Router::new()
            .route("/", get(hello_handler))
            .layer(SecurityHeadersLayer::new())
    }

    #[tokio::test]
    async fn test_x_content_type_options_header() {
        let app = create_test_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::X_CONTENT_TYPE_OPTIONS).map(|v| v.to_str().unwrap()),
            Some("nosniff")
        );
    }

    #[tokio::test]
    async fn test_x_frame_options_header() {
        let app = create_test_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::X_FRAME_OPTIONS).map(|v| v.to_str().unwrap()),
            Some("DENY")
        );
    }

    #[tokio::test]
    async fn test_x_xss_protection_header() {
        let app = create_test_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::X_XSS_PROTECTION).map(|v| v.to_str().unwrap()),
            Some("1; mode=block")
        );
    }

    #[tokio::test]
    async fn test_referrer_policy_header() {
        let app = create_test_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::REFERRER_POLICY).map(|v| v.to_str().unwrap()),
            Some("strict-origin-when-cross-origin")
        );
    }

    #[tokio::test]
    async fn test_content_security_policy_header() {
        let app = create_test_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let csp = response
            .headers()
            .get(header::CONTENT_SECURITY_POLICY)
            .map(|v| v.to_str().unwrap());

        assert!(csp.is_some());
        let csp_value = csp.unwrap();

        // Verify CSP contains expected directives
        assert!(csp_value.contains("default-src 'self'"));
        assert!(csp_value.contains("script-src 'self'"));
        assert!(csp_value.contains("style-src 'self'"));
        assert!(csp_value.contains("frame-ancestors 'none'"));
    }

    #[tokio::test]
    async fn test_all_security_headers_present() {
        let app = create_test_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify all security headers are present
        let headers = response.headers();
        assert!(headers.contains_key(header::X_CONTENT_TYPE_OPTIONS));
        assert!(headers.contains_key(header::X_FRAME_OPTIONS));
        assert!(headers.contains_key(header::X_XSS_PROTECTION));
        assert!(headers.contains_key(header::REFERRER_POLICY));
        assert!(headers.contains_key(header::CONTENT_SECURITY_POLICY));
    }

    #[tokio::test]
    async fn test_security_headers_on_error_response() {
        let app = Router::new()
            .route("/", get(|| async { StatusCode::INTERNAL_SERVER_ERROR }))
            .layer(SecurityHeadersLayer::new());

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Security headers should be present even on error responses
        assert!(response.headers().contains_key(header::X_CONTENT_TYPE_OPTIONS));
        assert!(response.headers().contains_key(header::X_FRAME_OPTIONS));
    }
}
