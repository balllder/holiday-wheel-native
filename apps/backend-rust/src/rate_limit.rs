//! Rate limiting middleware for API protection
//!
//! Provides configurable rate limiting using the token bucket algorithm
//! to prevent API abuse and ensure fair resource allocation.

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{
    net::SocketAddr,
    num::NonZeroU32,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

/// Rate limiter type alias
pub type GlobalRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Configuration for rate limiting
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub requests_per_second: u32,
    /// Burst capacity (allows temporary spikes)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 250,
        }
    }
}

impl RateLimitConfig {
    /// Create a stricter rate limit for auth endpoints
    pub fn auth() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 20,
        }
    }

    /// Create a more relaxed rate limit for general API endpoints
    pub fn api() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 250,
        }
    }

    /// Create rate limiter from this config
    pub fn to_limiter(&self) -> GlobalRateLimiter {
        let quota = Quota::per_second(NonZeroU32::new(self.requests_per_second).unwrap())
            .allow_burst(NonZeroU32::new(self.burst_size).unwrap());
        RateLimiter::direct(quota)
    }
}

/// Rate limiting layer for tower/axum
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<GlobalRateLimiter>,
}

impl RateLimitLayer {
    /// Create a new rate limit layer with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: Arc::new(config.to_limiter()),
        }
    }

    /// Create a rate limit layer with default configuration
    pub fn default_api() -> Self {
        Self::new(RateLimitConfig::api())
    }

    /// Create a stricter rate limit layer for auth endpoints
    pub fn for_auth() -> Self {
        Self::new(RateLimitConfig::auth())
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

/// Rate limiting service
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<GlobalRateLimiter>,
}

impl<S, ReqBody> Service<Request<ReqBody>> for RateLimitService<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let limiter = self.limiter.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Check rate limit
            match limiter.check() {
                Ok(_) => {
                    // Request allowed, proceed
                    inner.call(req).await
                }
                Err(_) => {
                    // Rate limit exceeded
                    let response = RateLimitExceeded.into_response();
                    Ok(response)
                }
            }
        })
    }
}

/// Response when rate limit is exceeded
struct RateLimitExceeded;

impl IntoResponse for RateLimitExceeded {
    fn into_response(self) -> Response<Body> {
        let body = serde_json::json!({
            "error": "rate_limit_exceeded",
            "message": "Too many requests. Please try again later.",
            "retry_after_seconds": 1
        });

        Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "application/json")
            .header("Retry-After", "1")
            .body(Body::from(body.to_string()))
            .unwrap()
    }
}

/// Create a default rate limiter for the API
pub fn create_api_rate_limiter() -> RateLimitLayer {
    RateLimitLayer::default_api()
}

/// Create a stricter rate limiter for authentication endpoints
pub fn create_auth_rate_limiter() -> RateLimitLayer {
    RateLimitLayer::for_auth()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_second, 100);
        assert_eq!(config.burst_size, 250);
    }

    #[test]
    fn test_auth_config() {
        let config = RateLimitConfig::auth();
        assert_eq!(config.requests_per_second, 10);
        assert_eq!(config.burst_size, 20);
    }

    #[test]
    fn test_api_config() {
        let config = RateLimitConfig::api();
        assert_eq!(config.requests_per_second, 100);
        assert_eq!(config.burst_size, 250);
    }

    #[test]
    fn test_create_limiter() {
        let config = RateLimitConfig::default();
        let _limiter = config.to_limiter();
        // Limiter created successfully
    }

    #[test]
    fn test_rate_limit_layer_creation() {
        let _layer = RateLimitLayer::default_api();
        let _layer = RateLimitLayer::for_auth();
        // Layers created successfully
    }

    #[tokio::test]
    async fn test_limiter_allows_within_limit() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 10,
        };
        let limiter = config.to_limiter();

        // Should allow burst_size requests immediately
        for _ in 0..10 {
            assert!(limiter.check().is_ok());
        }
    }

    #[tokio::test]
    async fn test_limiter_rejects_over_limit() {
        let config = RateLimitConfig {
            requests_per_second: 1,
            burst_size: 2,
        };
        let limiter = config.to_limiter();

        // First two should succeed (burst)
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());

        // Third should fail (over limit)
        assert!(limiter.check().is_err());
    }
}
