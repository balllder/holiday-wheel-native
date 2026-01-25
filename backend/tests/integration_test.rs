use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

/// Helper function to create a test database pool
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/project_starter_test".to_string());

    PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to create test pool")
}

/// Helper function to run migrations on test database
async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}

/// Helper function to clean up test database
async fn cleanup_database(pool: &PgPool) {
    sqlx::query("TRUNCATE items CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean up database");
}

#[tokio::test]
async fn test_create_item() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_database(&pool).await;

    let item_id = Uuid::new_v4();
    let item_name = "Test Item";
    let item_description = Some("Test Description");

    // Insert item
    let result = sqlx::query(
        "INSERT INTO items (id, name, description) VALUES ($1, $2, $3)"
    )
    .bind(item_id)
    .bind(item_name)
    .bind(item_description.clone())
    .execute(&pool)
    .await;

    assert!(result.is_ok());

    // Verify item was created
    let item: (Uuid, String, Option<String>) = sqlx::query_as(
        "SELECT id, name, description FROM items WHERE id = $1"
    )
    .bind(item_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch created item");

    assert_eq!(item.0, item_id);
    assert_eq!(item.1, item_name);
    assert_eq!(item.2, item_description);

    cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_list_items() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_database(&pool).await;

    // Create test items
    for i in 1..=3 {
        sqlx::query(
            "INSERT INTO items (id, name, description) VALUES ($1, $2, $3)"
        )
        .bind(Uuid::new_v4())
        .bind(format!("Item {}", i))
        .bind(Some(format!("Description {}", i)))
        .execute(&pool)
        .await
        .expect("Failed to create test item");
    }

    // List items
    let items: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, name FROM items ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to list items");

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].1, "Item 3"); // Most recent first

    cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_get_item_by_id() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_database(&pool).await;

    let item_id = Uuid::new_v4();
    let item_name = "Specific Item";

    // Create item
    sqlx::query(
        "INSERT INTO items (id, name) VALUES ($1, $2)"
    )
    .bind(item_id)
    .bind(item_name)
    .execute(&pool)
    .await
    .expect("Failed to create item");

    // Fetch item
    let item: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, name FROM items WHERE id = $1"
    )
    .bind(item_id)
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch item");

    assert!(item.is_some());
    let (id, name) = item.unwrap();
    assert_eq!(id, item_id);
    assert_eq!(name, item_name);

    cleanup_database(&pool).await;
}

#[tokio::test]
async fn test_get_nonexistent_item() {
    let pool = create_test_pool().await;
    run_migrations(&pool).await;
    cleanup_database(&pool).await;

    let nonexistent_id = Uuid::new_v4();

    let item: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM items WHERE id = $1"
    )
    .bind(nonexistent_id)
    .fetch_optional(&pool)
    .await
    .expect("Failed to query database");

    assert!(item.is_none());

    cleanup_database(&pool).await;
}
