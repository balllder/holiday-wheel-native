# OpenAPI Quick Start

## 🚀 In 60 Seconds

```bash
# Generate OpenAPI schema from backend
make openapi-generate

# Generate TypeScript client
make codegen-all

# View interactive docs (start backend first)
# http://localhost:3000/swagger-ui
```

## 📝 Adding a New Endpoint

### 1. Create Handler with Annotation

```rust
use utoipa::ToSchema;

// Define request/response models
#[derive(Serialize, Deserialize, ToSchema)]
pub struct MyRequest {
    #[schema(example = "value")]
    pub field: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MyResponse {
    #[schema(example = "result")]
    pub result: String,
}

// Annotate endpoint
#[utoipa::path(
    post,
    path = "/api/my-endpoint",
    request_body = MyRequest,
    responses(
        (status = 200, body = MyResponse),
        (status = 400, body = ErrorResponse),
    ),
    tag = "my-tag"
)]
pub async fn my_endpoint(
    Json(req): Json<MyRequest>,
) -> Result<Json<MyResponse>> {
    // Implementation
}
```

### 2. Register in lib.rs

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        // ... existing paths
        api::my_module::my_endpoint,  // ADD THIS
    ),
    components(
        schemas(
            // ... existing schemas
            api::my_module::MyRequest,   // ADD THIS
            api::my_module::MyResponse,  // ADD THIS
        )
    ),
    tags(
        (name = "my-tag", description = "My endpoints")  // ADD THIS
    )
)]
pub struct ApiDoc;
```

### 3. Generate & Commit

```bash
# Regenerate everything
make codegen-all

# Commit
git add backend/ frontend/src/api/
git commit -m "feat(api): add my-endpoint"
```

## 🔍 Pre-Commit Validation

The hook validates **3 things**:

1. ✅ **OpenAPI schema compiles**
   ```bash
   cargo build --bin generate-openapi
   ```

2. ✅ **Annotations are valid**
   ```bash
   cargo run --bin generate-openapi
   ```

3. ✅ **TypeScript client is current**
   ```bash
   make codegen-check
   ```

**If it fails:**
```bash
# Regenerate
make codegen-all

# Commit generated files
git add frontend/src/api/
git commit
```

## 📋 Required Annotations

### Models
```rust
#[derive(Serialize, Deserialize, ToSchema)]  // ← ToSchema required
#[schema(example = json!({"field": "value"}))]
pub struct MyModel {
    /// Field documentation
    #[schema(example = "value")]
    pub field: String,
}
```

### Endpoints
```rust
#[utoipa::path(         // ← Required
    post,               // HTTP method
    path = "/api/path", // Full path
    request_body = Req, // Request type
    responses(          // All status codes
        (status = 200, body = Resp),
        (status = 400, body = ErrorResponse),
    ),
    tag = "group"       // For grouping
)]
```

## 🧪 Testing

```bash
# 1. Start backend
cd backend && cargo run

# 2. Open Swagger UI
# http://localhost:3000/swagger-ui

# 3. Try the endpoints interactively
```

## 🐛 Troubleshooting

### Error: "no bin target named `generate-openapi`"

**Missing:** Binary registration in Cargo.toml

**Fix:**
```toml
[[bin]]
name = "generate-openapi"
path = "src/bin/generate-openapi.rs"
```

### Error: "ToSchema is not implemented"

**Missing:** ToSchema derive on model

**Fix:**
```rust
#[derive(Serialize, Deserialize, ToSchema)]  // Add ToSchema
pub struct MyModel { ... }
```

### Error: "OpenAPI schema validation failed"

**Debug:**
```bash
cd backend
cargo run --bin generate-openapi  # See full error
```

**Common:**
- Forgot `#[utoipa::path]` on new endpoint
- Model not registered in `lib.rs`

### Pre-Commit Hook Fails

**Check what changed:**
```bash
# Regenerate
make codegen-all

# See diff
git diff frontend/src/api/

# Commit if expected
git add frontend/src/api/
```

## 📚 Full Documentation

- **Implementation Guide:** [IMPLEMENTATION.md](./IMPLEMENTATION.md)
- **Backend Guide:** [backend/OPENAPI.md](../../backend/OPENAPI.md)
- **API Strategy:** [STRATEGY.md](./STRATEGY.md)
- **CLAUDE.md:** [Project Rules](../../CLAUDE.md#3-typescript-codegen-enforced)

## 🎯 Common Patterns

### Pagination
```rust
#[utoipa::path(
    params(
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),
    )
)]
```

### Path Parameters
```rust
#[utoipa::path(
    params(
        ("id" = Uuid, Path, description = "Resource ID")
    )
)]
```

### Authentication
```rust
#[utoipa::path(
    responses(
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(("bearer_auth" = []))
)]
```

## ✅ Checklist

- [ ] All models have `#[derive(ToSchema)]`
- [ ] All endpoints have `#[utoipa::path]`
- [ ] All endpoints registered in `lib.rs`
- [ ] All models registered in `lib.rs`
- [ ] Run: `make codegen-all`
- [ ] Commit generated files
- [ ] `make pre-commit` passes
