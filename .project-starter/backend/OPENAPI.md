# OpenAPI Implementation Guide

## Overview

This backend uses **utoipa** to automatically generate OpenAPI 3.0 schemas from Rust code annotations. This ensures type-safe API communication and automatic documentation generation.

## Architecture

```
Rust Code (utoipa annotations)
        ↓
OpenAPI Schema (generate-openapi binary)
        ↓
TypeScript Client (openapi-typescript)
        ↓
Frontend Code (type-safe API calls)
```

## Implementation

### 1. Dependencies (Cargo.toml)

```toml
[dependencies]
utoipa = { version = "4.2", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }
```

### 2. Annotating Models

Every DTO must derive `ToSchema`:

```rust
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Example Item"
}))]
pub struct Item {
    /// Unique identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,

    /// Item name
    #[schema(example = "Example Item", min_length = 1, max_length = 100)]
    pub name: String,
}
```

**Required:**
- `#[derive(ToSchema)]` on all request/response types
- Field-level documentation with `///`
- Example values with `#[schema(example = "...")]`
- Validation constraints (min_length, max_length, etc.)

### 3. Annotating Endpoints

Every API endpoint must have a `#[utoipa::path]` annotation:

```rust
#[utoipa::path(
    get,
    path = "/api/items/{id}",
    params(
        ("id" = Uuid, Path, description = "Item UUID")
    ),
    responses(
        (status = 200, description = "Item found", body = Item),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "items"
)]
pub async fn get_item(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Item>> {
    // Implementation
}
```

**Required:**
- HTTP method (get, post, put, delete)
- Path with parameters in curly braces
- All possible HTTP status codes
- Response body types for each status
- Tag for grouping endpoints

### 4. API Documentation Structure (lib.rs)

The `ApiDoc` struct defines the complete OpenAPI schema:

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Project Starter API",
        version = "0.1.0",
        description = "API description",
    ),
    paths(
        api::items::list_items,
        api::items::get_item,
        api::items::create_item,
    ),
    components(
        schemas(
            models::Item,
            models::CreateItemRequest,
            error::ErrorResponse,
        )
    ),
    tags(
        (name = "items", description = "Item management endpoints")
    )
)]
pub struct ApiDoc;
```

**When adding a new endpoint:**
1. Annotate the endpoint function with `#[utoipa::path]`
2. Add the function to `paths()` in `ApiDoc`
3. Add any new models to `components(schemas())`

### 5. Generating the Schema

```bash
# Generate OpenAPI JSON schema
cargo run --bin generate-openapi > openapi.json

# Or use the Makefile
make openapi-generate
```

**Output:** `frontend/src/api/openapi.json`

### 6. Swagger UI

The API includes interactive Swagger UI documentation:

- **URL:** http://localhost:3000/swagger-ui
- **Features:**
  - Browse all endpoints
  - Try API calls directly
  - View request/response schemas
  - See example values

Configured in `main.rs`:

```rust
use utoipa_swagger_ui::SwaggerUi;

let app = Router::new()
    // ... routes ...
    .merge(SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi()));
```

## Workflow

### Adding a New Endpoint

1. **Create handler function:**
   ```rust
   pub async fn my_endpoint() -> Json<MyResponse> {
       // Implementation
   }
   ```

2. **Add utoipa annotation:**
   ```rust
   #[utoipa::path(
       get,
       path = "/api/my-endpoint",
       responses(
           (status = 200, body = MyResponse)
       ),
       tag = "my-tag"
   )]
   pub async fn my_endpoint() -> Json<MyResponse> {
       // Implementation
   }
   ```

3. **Register in ApiDoc (lib.rs):**
   ```rust
   paths(
       // Existing paths...
       api::my_module::my_endpoint,
   ),
   components(
       schemas(
           // Existing schemas...
           models::MyResponse,
       )
   )
   ```

4. **Generate schema:**
   ```bash
   make codegen-all
   ```

5. **Commit generated files:**
   ```bash
   git add frontend/src/api/
   git commit -m "feat(api): add my-endpoint"
   ```

### Modifying an Existing Endpoint

1. **Update the handler and annotation**
2. **Regenerate schema:** `make codegen-all`
3. **Fix TypeScript errors** (breaking changes caught at compile-time)
4. **Commit changes**

## Validation & Enforcement

### Pre-Commit Hook

The pre-commit hook validates:

1. **OpenAPI schema compiles:** `cargo build --bin generate-openapi`
2. **Annotations are valid:** `cargo run --bin generate-openapi`
3. **TypeScript client is up-to-date:** `make codegen-check`

**If validation fails:**
```bash
# Fix annotations and regenerate
make codegen-all

# Commit generated files
git add frontend/src/api/
git commit
```

### CI/CD Pipeline

```bash
# In CI (runs automatically)
make codegen-check  # Fails if generated code is stale
```

## Troubleshooting

### Error: "no bin target named `generate-openapi`"

**Cause:** Binary not registered in Cargo.toml

**Fix:**
```toml
[[bin]]
name = "generate-openapi"
path = "src/bin/generate-openapi.rs"
```

### Error: "failed to resolve: use of undeclared crate or module `utoipa`"

**Cause:** Missing dependency

**Fix:**
```toml
[dependencies]
utoipa = { version = "4.2", features = ["axum_extras", "chrono", "uuid"] }
```

### Error: "ToSchema is not implemented for X"

**Cause:** Model missing `ToSchema` derive

**Fix:**
```rust
#[derive(Serialize, Deserialize, ToSchema)]  // Add ToSchema
pub struct X { ... }
```

### Pre-Commit Hook Fails

**Check generated schema:**
```bash
cd backend
cargo run --bin generate-openapi
```

**Common issues:**
- Missing `#[utoipa::path]` on new endpoints
- Missing `ToSchema` on new models
- Model not registered in `ApiDoc` in lib.rs

### TypeScript Errors After Regeneration

**This is EXPECTED!** TypeScript caught API contract changes.

1. Review the OpenAPI schema changes
2. Update frontend code to match new contract
3. Fix all TypeScript errors
4. Test the changes

## Best Practices

### 1. Complete Response Codes

**❌ BAD:**
```rust
#[utoipa::path(
    get,
    path = "/api/items",
    responses(
        (status = 200, body = Vec<Item>)
    )
)]
```

**✅ GOOD:**
```rust
#[utoipa::path(
    get,
    path = "/api/items",
    responses(
        (status = 200, description = "List of items", body = Vec<Item>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
```

### 2. Detailed Examples

**❌ BAD:**
```rust
#[derive(ToSchema)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
}
```

**✅ GOOD:**
```rust
#[derive(ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Example Item"
}))]
pub struct Item {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,

    #[schema(example = "Example Item", min_length = 1, max_length = 100)]
    pub name: String,
}
```

### 3. Descriptive Documentation

**❌ BAD:**
```rust
/// Get item
pub async fn get_item() -> Result<Json<Item>> {
```

**✅ GOOD:**
```rust
/// Get a specific item by ID
///
/// Retrieves detailed information about a single item.
/// Returns 404 if the item does not exist.
pub async fn get_item() -> Result<Json<Item>> {
```

## References

- [utoipa Documentation](https://docs.rs/utoipa/)
- [OpenAPI 3.0 Specification](https://swagger.io/specification/)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [Project API Codegen Strategy](../docs/api-codegen/STRATEGY.md)
