use std::sync::Arc;
use std::path::PathBuf;

use axum::{routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use rustls::crypto::ring::default_provider;
use socketioxide::SocketIo;
use tokio::sync::{OnceCell, RwLock};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod db;
mod email;
mod game;
mod routes;

use email::EmailService;
use game::GameManager;

/// Application state shared across handlers
pub struct AppState {
    pub game_manager: RwLock<GameManager>,
    pub db: db::Database,
    pub email: EmailService,
    pub io: OnceCell<SocketIo>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install the ring crypto provider for rustls
    default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database
    let db_path = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("DB_PATH"))
        .unwrap_or_else(|_| "puzzles.db".to_string());
    let db = db::Database::new(&db_path).await?;
    info!("Connected to database: {}", db_path);

    // Initialize email service
    let email = EmailService::from_env();
    info!(
        "Email service initialized (enabled: {})",
        std::env::var("EMAIL_ENABLED").unwrap_or_else(|_| "false".to_string())
    );

    // Initialize game manager
    let game_manager = GameManager::new();

    // Create shared state
    let state = Arc::new(AppState {
        game_manager: RwLock::new(game_manager),
        db,
        email,
        io: OnceCell::new(),
    });

    // Set up Socket.IO
    let (socket_layer, io) = SocketIo::builder()
        .with_state(state.clone())
        .build_layer();

    // Store SocketIo in state for background tasks
    state.io.set(io.clone()).expect("Failed to set SocketIo");

    // Register game socket handlers
    game::handlers::register_handlers(&io);

    // Build HTTP routes
    let app = Router::new()
        .route("/", get(routes::index))
        .route("/register", get(routes::register))
        .route("/lobby", get(routes::lobby))
        .route("/game", get(routes::game))
        .route("/admin", get(routes::admin))
        .route("/health", get(routes::health))
        .nest("/auth", auth::routes())
        .layer(socket_layer)
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse::<u16>()?;

    let ssl_enabled = std::env::var("SSL_ENABLED")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);
    let ssl_cert = std::env::var("SSL_CERT").ok();
    let ssl_key = std::env::var("SSL_KEY").ok();

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    // Use HTTPS if SSL is enabled and certificates are provided
    if ssl_enabled {
        match (ssl_cert, ssl_key) {
            (Some(cert_path), Some(key_path)) => {
                let config = RustlsConfig::from_pem_file(
                    PathBuf::from(&cert_path),
                    PathBuf::from(&key_path),
                )
                .await?;

                info!("Server running on https://0.0.0.0:{}", port);
                info!("SSL cert: {}, key: {}", cert_path, key_path);
                axum_server::bind_rustls(addr, config)
                    .serve(app.into_make_service())
                    .await?;
            }
            _ => {
                anyhow::bail!("SSL_ENABLED=true but SSL_CERT and/or SSL_KEY not set");
            }
        }
    } else {
        info!("Server running on http://0.0.0.0:{}", port);
        info!("To enable HTTPS, set SSL_ENABLED=true with SSL_CERT and SSL_KEY");
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}
