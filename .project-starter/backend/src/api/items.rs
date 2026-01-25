use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{CreateItemRequest, Item},
};

/// List all items
///
/// # Example
/// ```bash
/// curl http://localhost:3000/api/items
/// ```
#[utoipa::path(
    get,
    path = "/api/items",
    responses(
        (status = 200, description = "List of items", body = Vec<Item>),
        (status = 500, description = "Internal server error", body = crate::error::ErrorResponse)
    ),
    tag = "items"
)]
#[tracing::instrument(skip(pool))]
pub async fn list_items(State(pool): State<PgPool>) -> Result<Json<Vec<Item>>> {
    let items = sqlx::query_as::<_, Item>(
        "SELECT id, name, description, created_at FROM items ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    tracing::info!("Retrieved {} items", items.len());

    Ok(Json(items))
}

/// Get a specific item by ID
///
/// # Arguments
/// * `id` - UUID of the item
///
/// # Example
/// ```bash
/// curl http://localhost:3000/api/items/550e8400-e29b-41d4-a716-446655440000
/// ```
#[utoipa::path(
    get,
    path = "/api/items/{id}",
    params(
        ("id" = Uuid, Path, description = "Item UUID")
    ),
    responses(
        (status = 200, description = "Item found", body = Item),
        (status = 404, description = "Item not found", body = crate::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::ErrorResponse)
    ),
    tag = "items"
)]
#[tracing::instrument(skip(pool))]
pub async fn get_item(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Item>> {
    let item = sqlx::query_as::<_, Item>(
        "SELECT id, name, description, created_at FROM items WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    tracing::info!("Retrieved item: {}", item.id);

    Ok(Json(item))
}

/// Create a new item
///
/// # Request Body
/// ```json
/// {
///   "name": "Example Item",
///   "description": "Optional description"
/// }
/// ```
///
/// # Example
/// ```bash
/// curl -X POST http://localhost:3000/api/items \
///   -H "Content-Type: application/json" \
///   -d '{"name": "Example Item", "description": "A test item"}'
/// ```
#[utoipa::path(
    post,
    path = "/api/items",
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created successfully", body = Item),
        (status = 400, description = "Invalid input", body = crate::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::error::ErrorResponse)
    ),
    tag = "items"
)]
#[tracing::instrument(skip(pool))]
pub async fn create_item(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<Item>)> {
    // Validate input
    payload
        .validate()
        .map_err(AppError::InvalidInput)?;

    // Create item in database
    let item = sqlx::query_as::<_, Item>(
        "INSERT INTO items (id, name, description) VALUES ($1, $2, $3) RETURNING id, name, description, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(&payload.name)
    .bind(&payload.description)
    .fetch_one(&pool)
    .await?;

    tracing::info!("Created item: {}", item.id);

    Ok((StatusCode::CREATED, Json(item)))
}
