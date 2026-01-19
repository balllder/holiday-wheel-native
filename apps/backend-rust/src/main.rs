use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::path::PathBuf;
use std::time::Duration;

use axum::{middleware, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use rustls::crypto::ring::default_provider;
use socketioxide::SocketIo;
use tokio::sync::{OnceCell, RwLock};
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod db;
mod docs;
mod email;
mod game;
mod logging;
mod metrics;
mod rate_limit;
mod routes;
mod security;
pub mod validation;

use config::Config;
use email::EmailService;
use game::GameManager;
use metrics::MetricsState;
use rate_limit::{create_api_rate_limiter, create_auth_rate_limiter};
use security::SecurityHeadersLayer;

/// Shutdown signal timeout for connection draining
const SHUTDOWN_TIMEOUT_SECS: u64 = 30;

/// Creates a future that completes when a shutdown signal is received.
/// Handles both SIGTERM and SIGINT (Ctrl+C) on Unix systems.
/// On Windows, only Ctrl+C is handled.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C (SIGINT), initiating graceful shutdown...");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}

/// Application state shared across handlers
pub struct AppState {
    pub game_manager: RwLock<GameManager>,
    pub db: db::Database,
    pub email: EmailService,
    pub io: OnceCell<SocketIo>,
    /// Track socket IDs by user ID for session invalidation
    pub user_sockets: RwLock<HashMap<i64, HashSet<String>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install the ring crypto provider for rustls
    default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Initialize logging
    // Use JSON format when RUST_LOG_FORMAT=json (for production/structured logging)
    let env_filter = tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
    );

    let use_json = std::env::var("RUST_LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false);

    if use_json {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Initialize Prometheus metrics
    let metrics_state = MetricsState::new();
    info!("Prometheus metrics initialized");

    // Load and validate configuration from environment variables
    // This validates all required vars exist and have valid formats before proceeding
    let config = Config::from_env().map_err(|e| {
        tracing::error!("Configuration validation failed: {}", e);
        anyhow::anyhow!("Configuration error: {}", e)
    })?;

    // Log the loaded configuration (with sensitive values redacted)
    config.log_config();

    // Initialize database using validated config
    let db = db::Database::new(&config.database.path).await?;
    info!("Connected to database: {}", config.database.path);

    // Initialize email service using validated config
    let email = EmailService::new(email::EmailConfig {
        enabled: config.email.enabled,
        smtp_host: config.email.smtp_host.clone(),
        smtp_port: config.email.smtp_port,
        smtp_user: config.email.smtp_user.clone(),
        smtp_pass: config.email.smtp_pass.clone(),
        from_email: config.email.from_email.clone(),
        base_url: config.email.base_url.clone(),
    });
    info!("Email service initialized (enabled: {})", config.email.enabled);

    // Initialize game manager
    let game_manager = GameManager::new();

    // Create shared state
    let state = Arc::new(AppState {
        game_manager: RwLock::new(game_manager),
        db,
        email,
        io: OnceCell::new(),
        user_sockets: RwLock::new(HashMap::new()),
    });

    // Set up Socket.IO
    let (socket_layer, io) = SocketIo::builder()
        .with_state(state.clone())
        .build_layer();

    // Store SocketIo in state for background tasks
    state.io.set(io.clone()).expect("Failed to set SocketIo");

    // Register game socket handlers
    game::handlers::register_handlers(&io);

    // Spawn background task to clean up timed-out players
    {
        let state_clone = state.clone();
        let io_clone = io.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let mut manager = state_clone.game_manager.write().await;
                for (room_name, game) in manager.rooms.iter_mut() {
                    let removed = game.cleanup_timed_out_players();
                    if !removed.is_empty() {
                        info!(
                            "Removed {} timed-out player(s) from room {}: {:?}",
                            removed.len(),
                            room_name,
                            removed
                        );
                        // Broadcast updated state to room
                        let game_state = game.get_state();
                        io_clone.to(room_name.clone()).emit("state", &game_state).ok();
                    }
                }
            }
        });
    }

    // Build HTTP routes
    // Note: /docs routes don't need state, so we merge them separately
    // Auth routes get stricter rate limiting (10 req/s, 20 burst)
    let auth_routes = auth::routes().layer(create_auth_rate_limiter());

    // Create metrics routes (separate state)
    let metrics_routes = Router::new()
        .route("/metrics", get(metrics::metrics_handler))
        .with_state(metrics_state.clone());

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/register", get(routes::register))
        .route("/lobby", get(routes::lobby))
        .route("/game", get(routes::game))
        .route("/join", get(routes::join))
        .route("/admin", get(routes::admin))
        .route("/health", get(routes::health))
        .nest("/auth", auth_routes)
        .with_state(state)
        .merge(metrics_routes)
        .nest("/docs", docs::routes())
        .layer(socket_layer)
        .layer(CorsLayer::permissive())
        // Add metrics middleware to record request metrics
        .layer(middleware::from_fn_with_state(
            metrics_state,
            metrics::metrics_middleware,
        ))
        // Add security headers (X-Content-Type-Options, X-Frame-Options, CSP, etc.)
        .layer(SecurityHeadersLayer::new())
        // Apply general rate limiting (100 req/s, 250 burst) to all routes
        .layer(create_api_rate_limiter())
        // Add request ID middleware for structured logging (outermost layer = first to execute)
        .layer(middleware::from_fn(logging::request_id_middleware));

    // Start server using validated config
    let port = config.server.port;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    // Use HTTPS if SSL is enabled and certificates are provided (already validated by Config)
    if config.server.ssl_enabled {
        // SSL cert and key are guaranteed to exist when ssl_enabled is true (validated by Config)
        let cert_path = config.server.ssl_cert.as_ref().unwrap();
        let key_path = config.server.ssl_key.as_ref().unwrap();

        let tls_config = RustlsConfig::from_pem_file(
            PathBuf::from(cert_path),
            PathBuf::from(key_path),
        )
        .await?;

        info!("Server running on https://0.0.0.0:{}", port);
        info!("SSL cert: {}, key: {}", cert_path, key_path);
        info!("Prometheus metrics available at /metrics");
        info!("Graceful shutdown enabled ({}s drain timeout)", SHUTDOWN_TIMEOUT_SECS);

        // Create a handle for graceful shutdown
        let handle = axum_server::Handle::new();
        let shutdown_handle = handle.clone();

        // Spawn shutdown signal listener
        tokio::spawn(async move {
            shutdown_signal().await;
            info!("Allowing {}s for connections to drain...", SHUTDOWN_TIMEOUT_SECS);
            shutdown_handle.graceful_shutdown(Some(Duration::from_secs(SHUTDOWN_TIMEOUT_SECS)));
        });

        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await?;

        info!("Server shutdown complete");
    } else {
        info!("Server running on http://0.0.0.0:{}", port);
        info!("To enable HTTPS, set SSL_ENABLED=true with SSL_CERT and SSL_KEY");
        info!("Prometheus metrics available at /metrics");
        info!("Graceful shutdown enabled ({}s drain timeout)", SHUTDOWN_TIMEOUT_SECS);

        let listener = tokio::net::TcpListener::bind(addr).await?;

        // Create a channel to track when shutdown signal is received
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Use axum's serve with graceful shutdown
        // The server will stop accepting new connections when the signal fires,
        // then wait for existing connections to complete.
        let server = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_signal().await;
                let _ = shutdown_tx.send(());
                info!("Allowing {}s for connections to drain...", SHUTDOWN_TIMEOUT_SECS);
            });

        // Spawn timeout watcher that triggers force shutdown after drain period
        tokio::spawn(async move {
            // Wait for shutdown signal to be received
            let _ = shutdown_rx.await;
            // Then wait for the drain timeout
            tokio::time::sleep(Duration::from_secs(SHUTDOWN_TIMEOUT_SECS)).await;
            warn!("Drain timeout of {}s reached, forcing process exit", SHUTDOWN_TIMEOUT_SECS);
            std::process::exit(0);
        });

        server.await?;
        info!("Server shutdown complete");
    }

    Ok(())
}
