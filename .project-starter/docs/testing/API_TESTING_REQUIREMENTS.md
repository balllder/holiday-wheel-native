# API Testing Requirements

## Overview

While E2E tests validate complete user workflows through the UI, **API integration tests are mandatory** to ensure backend endpoints work correctly at the HTTP contract level.

## Why API-Level Tests Matter

E2E tests exercise the full stack but:
- May not cover all API error cases
- Don't validate HTTP status codes directly
- Don't test API-specific concerns (rate limiting, authentication headers, etc.)

**Integration tests fill this gap by testing the backend API directly.**

---

## API Integration Test Requirements

### MANDATORY Coverage

Every backend API endpoint MUST have integration tests covering:

1. **Happy path:**
   - Valid request succeeds
   - Correct HTTP status code (200, 201, 204)
   - Response body matches OpenAPI schema
   - Database state reflects the operation

2. **Error cases:**
   - Invalid input (400 Bad Request)
   - Authentication failures (401 Unauthorized)
   - Authorization failures (403 Forbidden)
   - Resource not found (404 Not Found)
   - Conflicts (409 Conflict)
   - Server errors (500 Internal Server Error)

3. **Edge cases:**
   - Boundary conditions (max length, min/max values)
   - Special characters in input
   - Concurrent requests
   - Race conditions

4. **Data persistence:**
   - Verify database state after operations
   - Test cascading deletes
   - Test transaction rollbacks on errors

---

## Test Structure

### Rust Example (Axum + SQLx)

```rust
// backend/tests/integration/user_api_test.rs
#[tokio::test]
async fn test_create_user_success() {
    let pool = setup_test_db().await;
    let app = create_app(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"test@example.com","name":"Test User"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let user: User = serde_json::from_slice(&body).unwrap();

    assert_eq!(user.email, "test@example.com");

    // Verify database
    let db_user = User::find_by_email(&pool, "test@example.com")
        .await
        .unwrap();
    assert_eq!(db_user.email, "test@example.com");

    cleanup_test_db(pool).await;
}

#[tokio::test]
async fn test_create_user_duplicate_email() {
    let pool = setup_test_db().await;
    let app = create_app(pool.clone());

    // Create first user
    User::create(&pool, "test@example.com", "Test User").await.unwrap();

    // Attempt duplicate
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"test@example.com","name":"Test User"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    cleanup_test_db(pool).await;
}
```

### TypeScript Example (Express + Supertest)

```typescript
// backend/tests/integration/users.test.ts
import request from 'supertest';
import { app } from '../../src/app';
import { db } from '../../src/db';

describe('POST /api/v1/users', () => {
  beforeEach(async () => {
    await db.migrate.latest();
  });

  afterEach(async () => {
    await db.migrate.rollback();
  });

  test('creates user successfully', async () => {
    const response = await request(app)
      .post('/api/v1/users')
      .send({
        email: 'test@example.com',
        name: 'Test User',
      })
      .expect(201);

    expect(response.body).toMatchObject({
      email: 'test@example.com',
      name: 'Test User',
    });

    // Verify database
    const user = await db('users')
      .where({ email: 'test@example.com' })
      .first();
    expect(user).toBeDefined();
    expect(user.name).toBe('Test User');
  });

  test('rejects invalid email', async () => {
    const response = await request(app)
      .post('/api/v1/users')
      .send({
        email: 'not-an-email',
        name: 'Test User',
      })
      .expect(400);

    expect(response.body.error).toMatch(/invalid email/i);
  });

  test('rejects duplicate email', async () => {
    await db('users').insert({
      email: 'test@example.com',
      name: 'Existing User',
    });

    const response = await request(app)
      .post('/api/v1/users')
      .send({
        email: 'test@example.com',
        name: 'Test User',
      })
      .expect(409);

    expect(response.body.error).toMatch(/already exists/i);
  });
});

describe('GET /api/v1/users/:id', () => {
  test('returns user by id', async () => {
    const [userId] = await db('users').insert({
      email: 'test@example.com',
      name: 'Test User',
    }).returning('id');

    const response = await request(app)
      .get(`/api/v1/users/${userId}`)
      .expect(200);

    expect(response.body).toMatchObject({
      id: userId,
      email: 'test@example.com',
      name: 'Test User',
    });
  });

  test('returns 404 for non-existent user', async () => {
    const response = await request(app)
      .get('/api/v1/users/00000000-0000-0000-0000-000000000000')
      .expect(404);

    expect(response.body.error).toMatch(/not found/i);
  });
});

describe('PUT /api/v1/users/:id', () => {
  test('updates user', async () => {
    const [userId] = await db('users').insert({
      email: 'test@example.com',
      name: 'Test User',
    }).returning('id');

    const response = await request(app)
      .put(`/api/v1/users/${userId}`)
      .send({
        name: 'Updated Name',
      })
      .expect(200);

    expect(response.body.name).toBe('Updated Name');

    // Verify database
    const user = await db('users').where({ id: userId }).first();
    expect(user.name).toBe('Updated Name');
  });
});

describe('DELETE /api/v1/users/:id', () => {
  test('deletes user', async () => {
    const [userId] = await db('users').insert({
      email: 'test@example.com',
      name: 'Test User',
    }).returning('id');

    await request(app)
      .delete(`/api/v1/users/${userId}`)
      .expect(204);

    // Verify database
    const user = await db('users').where({ id: userId }).first();
    expect(user).toBeUndefined();
  });
});
```

---

## API Contract Testing

### Validate OpenAPI Schema Matches Implementation

```typescript
// backend/tests/integration/openapi-contract.test.ts
import request from 'supertest';
import { app } from '../../src/app';
import Ajv from 'ajv';
import openApiSchema from '../../../documentation/static/api/openapi.json';

const ajv = new Ajv();

describe('OpenAPI Contract Validation', () => {
  test('POST /api/v1/users response matches schema', async () => {
    const response = await request(app)
      .post('/api/v1/users')
      .send({
        email: 'test@example.com',
        name: 'Test User',
      });

    // Get schema for this endpoint
    const schema = openApiSchema.paths['/api/v1/users'].post.responses['201'].content['application/json'].schema;

    const validate = ajv.compile(schema);
    const valid = validate(response.body);

    expect(valid).toBe(true);
    if (!valid) {
      console.error('Schema validation errors:', validate.errors);
    }
  });
});
```

---

## Authentication & Authorization Testing

### Test Protected Endpoints

```typescript
describe('Authentication', () => {
  test('rejects unauthenticated requests', async () => {
    await request(app)
      .get('/api/v1/users/me')
      .expect(401);
  });

  test('accepts valid JWT token', async () => {
    const token = generateValidToken({ userId: 'test-user-id' });

    await request(app)
      .get('/api/v1/users/me')
      .set('Authorization', `Bearer ${token}`)
      .expect(200);
  });

  test('rejects expired JWT token', async () => {
    const token = generateExpiredToken({ userId: 'test-user-id' });

    await request(app)
      .get('/api/v1/users/me')
      .set('Authorization', `Bearer ${token}`)
      .expect(401);
  });
});

describe('Authorization', () => {
  test('allows user to access own data', async () => {
    const token = generateValidToken({ userId: 'user-123' });

    await request(app)
      .get('/api/v1/users/user-123')
      .set('Authorization', `Bearer ${token}`)
      .expect(200);
  });

  test('prevents user from accessing other user data', async () => {
    const token = generateValidToken({ userId: 'user-123' });

    await request(app)
      .get('/api/v1/users/user-456')
      .set('Authorization', `Bearer ${token}`)
      .expect(403);
  });

  test('allows admin to access all user data', async () => {
    const token = generateValidToken({ userId: 'admin-1', role: 'admin' });

    await request(app)
      .get('/api/v1/users/user-456')
      .set('Authorization', `Bearer ${token}`)
      .expect(200);
  });
});
```

---

## Performance Testing

### Test API Response Times

```typescript
describe('Performance', () => {
  test('GET /api/v1/users responds within 100ms', async () => {
    const start = Date.now();

    await request(app)
      .get('/api/v1/users')
      .expect(200);

    const duration = Date.now() - start;
    expect(duration).toBeLessThan(100);
  });

  test('handles 100 concurrent requests', async () => {
    const requests = Array.from({ length: 100 }, () =>
      request(app).get('/api/v1/users').expect(200)
    );

    await Promise.all(requests);
  });
});
```

---

## Makefile Integration

```makefile
# Run API integration tests
test-api: ## Run API integration tests
	@cd backend && cargo test --test '*' || npm test -- tests/integration/

# Run with coverage
test-api-coverage: ## Run API tests with coverage
	@cd backend && cargo tarpaulin --test '*' || npm test -- tests/integration/ --coverage

# Run specific API test
test-api-users: ## Run user API tests
	@cd backend && cargo test integration::user_api_test || npm test -- tests/integration/users.test.ts
```

---

## Coverage Requirements

**Each API endpoint MUST have:**
- ✅ At least 1 happy path test
- ✅ At least 2 error case tests
- ✅ Database state verification
- ✅ OpenAPI schema validation (optional but recommended)

**Failure to meet these requirements blocks PR merge.**

---

## CI/CD Integration

```yaml
# .github/workflows/api-tests.yml
name: API Integration Tests

on:
  pull_request:
    paths:
      - 'backend/**'

jobs:
  api-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start database
        run: docker-compose up -d postgres

      - name: Run API integration tests
        run: make test-api

      - name: Check coverage
        run: make test-api-coverage
```

---

## Relationship to E2E Tests

| Test Type | What It Tests | How It Tests |
|-----------|---------------|--------------|
| **API Integration** | Backend HTTP API contracts | Direct HTTP requests to endpoints |
| **E2E** | Complete user workflows | Browser interactions through UI |

**Both are required:**
- API integration tests ensure the backend API works correctly
- E2E tests ensure the full application (UI + API) works for users

**Example:**
- **API Test:** POST /api/v1/bookings returns 201 with correct JSON
- **E2E Test:** User fills booking form → clicks "Book Now" → sees confirmation page

---

## Success Metrics

API integration testing is successful when:

- ✅ Every API endpoint has integration tests
- ✅ Coverage ≥ 80% for backend API code
- ✅ All error cases tested (4xx, 5xx)
- ✅ Authentication/authorization tested
- ✅ Database state verified in tests
- ✅ Tests run in < 3 minutes
- ✅ Flakiness rate < 1%

---

## Related Documents

- [Testing Guide](./TESTING_GUIDE.md) - Overall testing strategy
- [E2E Testing](./E2E_TESTING.md) - E2E testing best practices
- [API Codegen Strategy](../api-codegen/STRATEGY.md) - OpenAPI generation

---

**Remember: API integration tests are not optional. Every endpoint must be tested.**
