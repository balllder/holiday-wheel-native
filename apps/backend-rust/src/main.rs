use std::sync::Arc;

use axum::{routing::get, Router};
use socketioxide::SocketIo;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod db;
mod game;
mod routes;

use game::GameManager;

/// Application state shared across handlers
pub struct AppState {
    pub game_manager: RwLock<GameManager>,
    pub db: db::Database,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // Initialize game manager
    let game_manager = GameManager::new();

    // Create shared state
    let state = Arc::new(AppState {
        game_manager: RwLock::new(game_manager),
        db,
    });

    // Set up Socket.IO
    let (socket_layer, io) = SocketIo::builder()
        .with_state(state.clone())
        .build_layer();

    // Register game socket handlers
    game::handlers::register_handlers(&io);

    // Build HTTP routes
    let app = Router::new()
        .route("/health", get(routes::health))
        .nest("/auth", auth::routes())
        .layer(socket_layer)
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "5000".to_string())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!("Server running on http://0.0.0.0:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}
