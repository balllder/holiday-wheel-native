use sqlx::{postgres::PgPoolOptions, PgPool};

/// Create a PostgreSQL connection pool
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string
///
/// # Returns
/// A configured connection pool
///
/// # Errors
/// Returns an error if the connection fails
pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    tracing::info!("Database connection pool created successfully");

    Ok(pool)
}
