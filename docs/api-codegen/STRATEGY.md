# API Code Generation Strategy

## Overview

**Automatically generate TypeScript clients from backend API schemas** to ensure type-safe API communication with zero manual maintenance.

## The Problem

Without code generation:
- ❌ Frontend API calls are manually written
- ❌ No type safety between frontend and backend
- ❌ API contract changes break frontend silently
- ❌ Manual API documentation becomes stale
- ❌ Duplicated type definitions
- ❌ Integration errors caught at runtime

## The Solution

**Single Source of Truth:** Backend code → OpenAPI schema → TypeScript client

```
Backend Code (annotated)
        ↓
OpenAPI Schema (generated)
        ↓
TypeScript Client (generated)
        ↓
Frontend Code (type-safe)
```

---

## Implementation Workflow

### Step 1: Annotate Backend Routes

**Rust Example (utoipa):**
```rust
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct User {
    #[schema(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub id: Uuid,

    #[schema(example = "user@example.com")]
    pub email: String,

    #[schema(example = "John Doe")]
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
    ),
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    tag = "users"
)]
pub async fn get_user(
    Path(id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    // ...
}
```

**Python Example (FastAPI):**
```python
from pydantic import BaseModel
from fastapi import FastAPI

class User(BaseModel):
    id: UUID
    email: str
    name: str

@app.get("/api/v1/users/{user_id}", response_model=User)
async def get_user(user_id: UUID):
    # ...
```

**Node.js Example (TypeScript + tsoa):**
```typescript
import { Get, Route, Path, Response } from 'tsoa';

interface User {
  id: string;
  email: string;
  name: string;
}

@Route('api/v1/users')
export class UsersController {
  @Get('{userId}')
  @Response<404>('Not Found')
  public async getUser(@Path() userId: string): Promise<User> {
    // ...
  }
}
```

---

### Step 2: Generate OpenAPI Schema

```bash
make openapi-generate
```

**Output:** `documentation/static/api/openapi.json`

**Rust (utoipa):**
```rust
// backend/src/bin/generate-openapi.rs
use utoipa::OpenApi;
use my_api::ApiDoc;

fn main() {
    let openapi = ApiDoc::openapi();
    println!("{}", openapi.to_pretty_json().unwrap());
}
```

**Python (FastAPI):**
```python
# Auto-generated at runtime
# curl http://localhost:8000/openapi.json > openapi.json
```

**Node.js (tsoa):**
```bash
tsoa spec-and-routes
```

---

### Step 3: Generate TypeScript Client

```bash
make codegen-frontend
```

**Tools:**
- **openapi-typescript** - Generates TypeScript types
- **openapi-fetch** - Type-safe fetch wrapper

**Installation:**
```bash
cd frontend
npm install --save-dev openapi-typescript
npm install openapi-fetch
```

**Script (package.json):**
```json
{
  "scripts": {
    "codegen": "openapi-typescript ../documentation/static/api/openapi.json -o src/api/schema.d.ts"
  }
}
```

**Output:** `frontend/src/api/schema.d.ts`

---

### Step 4: Create Type-Safe API Client

**frontend/src/api/client.ts:**
```typescript
import createClient from 'openapi-fetch';
import type { paths } from './schema';

export const apiClient = createClient<paths>({
  baseUrl: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000',
});

// Add authentication interceptor
apiClient.use({
  async onRequest({ request }) {
    const token = localStorage.getItem('auth_token');
    if (token) {
      request.headers.set('Authorization', `Bearer ${token}`);
    }
    return request;
  },
});

// Add error interceptor
apiClient.use({
  async onResponse({ response }) {
    if (!response.ok) {
      const error = await response.json();
      throw new ApiError(response.status, error.message);
    }
    return response;
  },
});
```

---

### Step 5: Use in Frontend Components

**Type-safe API calls with auto-complete:**

```typescript
import { apiClient } from '@/api/client';

// GET request
const { data, error } = await apiClient.GET('/api/v1/users/{id}', {
  params: {
    path: { id: userId },
  },
});

if (error) {
  console.error('Failed to fetch user:', error);
  return;
}

// data is fully typed as User
console.log(data.name); // TypeScript knows this field exists

// POST request
const { data: newUser, error: createError } = await apiClient.POST('/api/v1/users', {
  body: {
    email: 'user@example.com',
    name: 'John Doe',
  },
});

// PUT request
const { data: updated } = await apiClient.PUT('/api/v1/users/{id}', {
  params: {
    path: { id: userId },
  },
  body: {
    name: 'Jane Doe',
  },
});

// DELETE request
await apiClient.DELETE('/api/v1/users/{id}', {
  params: {
    path: { id: userId },
  },
});
```

---

## Makefile Integration

**Root Makefile:**
```makefile
# OpenAPI Schema Generation
openapi-generate: ## Generate OpenAPI schema from backend
	@echo "Generating OpenAPI schema..."
	cd backend && cargo run --bin generate-openapi > ../documentation/static/api/openapi.json
	@echo "OpenAPI schema generated ✓"

openapi-validate: openapi-generate ## Validate OpenAPI schema
	@echo "Validating OpenAPI schema..."
	npx @redocly/cli lint documentation/static/api/openapi.json
	@echo "OpenAPI schema is valid ✓"

# TypeScript Client Generation
codegen-frontend: openapi-generate ## Generate TypeScript client
	@echo "Generating TypeScript client..."
	cd frontend && npm run codegen
	@echo "TypeScript client generated ✓"

codegen-all: openapi-generate codegen-frontend ## Generate everything
	@echo "Code generation complete ✓"

codegen-check: ## Check if generated code is up-to-date
	@make codegen-all
	@if git diff --quiet documentation/static/api/openapi.json frontend/src/api/schema.d.ts; then \
		echo "Generated code is up-to-date ✓"; \
	else \
		echo "ERROR: Generated code is out of date. Run 'make codegen-all'"; \
		exit 1; \
	fi
```

---

## Development Workflow

### When Adding a New API Endpoint

1. **Write backend handler with annotations**
2. **Regenerate everything:**
   ```bash
   make codegen-all
   ```
3. **Use type-safe client in frontend**
4. **Commit both source and generated files**

### Pre-Commit Hook

Add to `.git/hooks/pre-commit`:
```bash
#!/bin/sh
# Check if generated code is up-to-date
make codegen-check || {
    echo "Error: Generated code is out of date"
    echo "Run 'make codegen-all' and commit the changes"
    exit 1
}
```

---

## CI/CD Integration

**GitHub Actions (disabled by default):**

```yaml
name: Check Generated Code

on:
  pull_request:
    paths:
      - 'backend/**'
      - 'frontend/src/api/**'

jobs:
  check-codegen:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup backend toolchain
      - name: Setup Node.js
      - name: Install dependencies
      - name: Check generated code
        run: make codegen-check
```

---

## Technology Stack by Language

### Rust
- **OpenAPI Generation:** utoipa
- **Swagger UI:** utoipa-swagger-ui
- **Frontend:** openapi-typescript + openapi-fetch

### Python (FastAPI)
- **OpenAPI Generation:** Built-in (Pydantic models)
- **Swagger UI:** Built-in at `/docs`
- **Frontend:** openapi-typescript + openapi-fetch

### Node.js (TypeScript)
- **OpenAPI Generation:** tsoa or NestJS
- **Swagger UI:** swagger-ui-express
- **Frontend:** openapi-typescript + openapi-fetch

### Go
- **OpenAPI Generation:** swaggo/swag
- **Swagger UI:** Built with swag
- **Frontend:** openapi-typescript + openapi-fetch

### Java (Spring Boot)
- **OpenAPI Generation:** springdoc-openapi
- **Swagger UI:** springdoc-openapi-ui
- **Frontend:** openapi-typescript + openapi-fetch

---

## Benefits

### 1. Type Safety
```typescript
// ✅ TypeScript knows the structure
const { data } = await apiClient.GET('/api/v1/users/{id}', {
  params: { path: { id: '123' } }
});

console.log(data.name); // ✓ TypeScript knows this exists

// ❌ Compile-time error
console.log(data.nonexistent); // Error: Property 'nonexistent' does not exist
```

### 2. Auto-Complete
IDE provides suggestions for:
- Available endpoints
- Request parameters
- Request body shape
- Response structure

### 3. Refactoring Safety
```rust
// Backend: Rename field
pub struct User {
    pub full_name: String, // was: name
}
```

After `make codegen-all`:
```typescript
// Frontend: TypeScript errors show where to update
console.log(data.name); // Error: Property 'name' does not exist
console.log(data.full_name); // ✓ Works
```

### 4. Always Up-to-Date Documentation
- Swagger UI at `/docs/swagger` (live documentation)
- API reference in Docusaurus (static docs)
- OpenAPI schema downloadable for external teams

### 5. No Manual API Client Code
```typescript
// Before: Manual fetch calls
const response = await fetch(`/api/v1/users/${id}`);
const data = await response.json(); // any type ❌

// After: Generated client
const { data } = await apiClient.GET('/api/v1/users/{id}', {
  params: { path: { id } }
}); // data is User type ✓
```

---

## Common Patterns

### Authentication
```typescript
// Add token to all requests
apiClient.use({
  async onRequest({ request }) {
    const token = getAuthToken();
    if (token) {
      request.headers.set('Authorization', `Bearer ${token}`);
    }
    return request;
  },
});
```

### Error Handling
```typescript
apiClient.use({
  async onResponse({ response }) {
    if (!response.ok) {
      const error = await response.json();
      throw new ApiError(response.status, error);
    }
    return response;
  },
});
```

### Logging
```typescript
apiClient.use({
  async onRequest({ request }) {
    console.log(`${request.method} ${request.url}`);
    return request;
  },
});
```

---

## Troubleshooting

### Generated Code Out of Sync

```bash
# Check what changed
make codegen-all
git diff

# If expected, commit
git add -A
git commit -m "chore: update generated API client"
```

### OpenAPI Schema Not Generating

**Rust:**
```bash
# Check if binary exists
cd backend
cargo run --bin generate-openapi

# If error, verify bin/generate-openapi.rs is in Cargo.toml
```

**Python:**
```bash
# FastAPI serves schema at /openapi.json
curl http://localhost:8000/openapi.json > openapi.json
```

### TypeScript Errors After Regeneration

**This is a good thing!** It means:
1. API contract changed
2. TypeScript caught the breaking change
3. You need to update frontend code

Fix the errors, don't ignore them.

---

## External Team Integration

Share the OpenAPI schema with external teams:

```bash
# Download schema
curl https://api.example.com/docs/openapi.json > openapi.json

# Generate client for their language
# Python
openapi-generator generate -i openapi.json -g python

# Java
openapi-generator generate -i openapi.json -g java

# Go
openapi-generator generate -i openapi.json -g go
```

---

## Success Metrics

After implementing API code generation:

- ✅ Zero manual API client code
- ✅ Type errors caught at compile-time
- ✅ API documentation always up-to-date
- ✅ Refactoring is safe (TypeScript catches breaks)
- ✅ Faster frontend development (auto-complete)
- ✅ Fewer runtime errors
- ✅ Integration teams have accurate schema

---

## Next Steps

1. **Implement OpenAPI generation** in backend
2. **Set up TypeScript client generation** in frontend
3. **Add Makefile commands** for code generation
4. **Configure pre-commit hook** to check generated code
5. **Train team** on new workflow

---

**Remember: Generated code is not a luxury, it's a necessity for type-safe development.**
