# Testing Guide

## Testing Philosophy

**Quality is not negotiable. Tests are written BEFORE implementation.**

This guide covers the complete testing strategy for all projects built with this template.

---

## Test Pyramid

```
        /\
       /  \
      / E2E \       10-20% of tests
     /------\
    /        \
   / INTEGRA \     30-40% of tests
  /    TION   \
 /------------\
/              \
/     UNIT      \   40-50% of tests
/________________\
```

### Unit Tests (40-50%)
- Test individual functions/methods in isolation
- Fast (milliseconds per test)
- No external dependencies (mock everything)
- High code coverage (≥80%)

### Integration Tests (30-40%)
- Test multiple components working together
- Test API endpoints with real database
- Test service layer interactions
- Moderate speed (seconds per test)

**See [API Testing Requirements](./API_TESTING_REQUIREMENTS.md) for mandatory API testing standards.**

### E2E Tests (10-20%)
- Test complete user workflows from UI
- Test full stack (frontend + backend + database)
- Slow (seconds to minutes per test)
- Focus on critical user journeys

**See [E2E Testing Best Practices](./E2E_TESTING.md) for detailed E2E guidance.**

---

## Test-Driven Development (TDD)

### Red-Green-Refactor Cycle

```
1. RED:    Write failing test
2. GREEN:  Write minimal code to pass
3. REFACTOR: Clean up code
4. REPEAT
```

### E2E-First Development

For user stories, write E2E test FIRST:

```typescript
// 1. Write E2E test from acceptance criteria (RED)
test('User can register', async ({ page }) => {
  await page.goto('/register');
  await page.fill('[name="email"]', 'user@example.com');
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/dashboard');
});
// Test FAILS - feature doesn't exist

// 2. Implement feature (GREEN)
// - Backend API
// - Frontend form
// - Validation
// Test PASSES

// 3. Refactor (still GREEN)
// - Clean up code
// - Extract components
// Test still PASSES
```

---

## Coverage Requirements

### Minimum Coverage: 80%

```bash
# Check coverage
make test-coverage

# Coverage report
make test-coverage-report
```

### What to Cover

**✅ MUST Cover (100%):**
- Business logic
- Data transformations
- Authentication/authorization
- Payment processing
- Security-critical code

**✅ SHOULD Cover (80%+):**
- API endpoints
- Database queries
- Form validation
- Error handling

**⚠️ MAY Skip:**
- Simple getters/setters
- Trivial utility functions
- Generated code
- Third-party library wrappers

---

## Unit Testing

### Backend Unit Tests

**Rust Example:**
```rust
// backend/src/models/user.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_email_validation() {
        let valid_email = "user@example.com";
        assert!(User::validate_email(valid_email).is_ok());

        let invalid_email = "not-an-email";
        assert!(User::validate_email(invalid_email).is_err());
    }

    #[test]
    fn test_password_hashing() {
        let password = "SecurePass123!";
        let hashed = User::hash_password(password).unwrap();

        assert_ne!(password, hashed);
        assert!(User::verify_password(password, &hashed).unwrap());
    }
}
```

**TypeScript/JavaScript Example:**
```typescript
// frontend/src/utils/validation.test.ts
import { validateEmail, validatePassword } from './validation';

describe('Email Validation', () => {
  test('accepts valid email', () => {
    expect(validateEmail('user@example.com')).toBe(true);
  });

  test('rejects invalid email', () => {
    expect(validateEmail('not-an-email')).toBe(false);
  });
});

describe('Password Validation', () => {
  test('accepts strong password', () => {
    expect(validatePassword('SecurePass123!')).toBe(true);
  });

  test('rejects weak password', () => {
    expect(validatePassword('123')).toBe(false);
  });
});
```

### Frontend Component Tests

```typescript
// frontend/src/components/LoginForm.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { LoginForm } from './LoginForm';

describe('LoginForm', () => {
  test('renders email and password fields', () => {
    render(<LoginForm />);

    expect(screen.getByLabelText('Email')).toBeInTheDocument();
    expect(screen.getByLabelText('Password')).toBeInTheDocument();
  });

  test('shows validation error for invalid email', async () => {
    render(<LoginForm />);

    const emailInput = screen.getByLabelText('Email');
    fireEvent.change(emailInput, { target: { value: 'invalid-email' } });
    fireEvent.blur(emailInput);

    expect(await screen.findByText('Invalid email address')).toBeInTheDocument();
  });

  test('calls onSubmit with email and password', () => {
    const onSubmit = jest.fn();
    render(<LoginForm onSubmit={onSubmit} />);

    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'user@example.com' },
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'SecurePass123!' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Login' }));

    expect(onSubmit).toHaveBeenCalledWith({
      email: 'user@example.com',
      password: 'SecurePass123!',
    });
  });
});
```

---

## Integration Testing

### Backend API Integration Tests (MANDATORY)

**Test API endpoints with real database:**

Every API endpoint MUST have integration tests covering:
- ✅ Happy path (200/201 success)
- ✅ Error cases (400/401/403/404/409/500)
- ✅ Database state verification
- ✅ Request/response contract validation

**See [API Testing Requirements](./API_TESTING_REQUIREMENTS.md) for complete coverage requirements.**

```rust
// backend/tests/integration/user_registration_test.rs
#[tokio::test]
async fn test_user_registration() {
    // Setup test database
    let pool = setup_test_db().await;
    let app = create_app(pool.clone());

    // Make request
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"email":"user@example.com","password":"SecurePass123!"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert response
    assert_eq!(response.status(), StatusCode::CREATED);

    // Assert database state
    let user = User::find_by_email(&pool, "user@example.com")
        .await
        .unwrap();
    assert_eq!(user.email, "user@example.com");

    // Cleanup
    cleanup_test_db(pool).await;
}
```

**TypeScript Example (Node.js + Express):**
```typescript
// backend/tests/integration/auth.test.ts
import request from 'supertest';
import { app } from '../../src/app';
import { db } from '../../src/db';

describe('POST /api/v1/auth/register', () => {
  beforeEach(async () => {
    await db.migrate.latest();
  });

  afterEach(async () => {
    await db.migrate.rollback();
  });

  test('registers new user', async () => {
    const response = await request(app)
      .post('/api/v1/auth/register')
      .send({
        email: 'user@example.com',
        password: 'SecurePass123!',
      });

    expect(response.status).toBe(201);
    expect(response.body).toMatchObject({
      user: {
        email: 'user@example.com',
      },
    });

    // Verify database
    const user = await db('users').where({ email: 'user@example.com' }).first();
    expect(user).toBeDefined();
  });

  test('rejects duplicate email', async () => {
    // Create existing user
    await request(app).post('/api/v1/auth/register').send({
      email: 'user@example.com',
      password: 'Pass123!',
    });

    // Attempt duplicate
    const response = await request(app)
      .post('/api/v1/auth/register')
      .send({
        email: 'user@example.com',
        password: 'Pass456!',
      });

    expect(response.status).toBe(409);
    expect(response.body.error).toMatch(/already exists/i);
  });
});
```

---

## E2E Testing

**See [E2E Testing Best Practices](./E2E_TESTING.md) for complete guide.**

**MANDATORY Requirements:**
- **Use the UI exposed to end users** (no direct API calls)
- **Cover complete lifecycle steps** (full workflows from start to finish)
- **Use real services by default** (mock only when cannot deploy internally OR incurs extra costs)
- **Document all mocking decisions** (explicit review required)

Quick summary:
- Test COMPLETE user lifecycles through browser UI
- Test full stack integration
- Complete test isolation (own database state)
- Real-looking test data
- Use real services (mock only with documented justification)
- Include mobile testing (80/20 rule)
- Include performance assertions
- Visual/UI checks included
- Write BEFORE implementation (TDD)

---

## Test Data Management

### Test Factories

**Create reusable test data factories:**

```typescript
// tests/factories/user.factory.ts
import { faker } from '@faker-js/faker';

export function createTestUser(overrides = {}) {
  return {
    id: faker.string.uuid(),
    email: faker.internet.email(),
    firstName: faker.person.firstName(),
    lastName: faker.person.lastName(),
    phone: faker.phone.number(),
    createdAt: faker.date.recent(),
    ...overrides,
  };
}

export function createTestProperty(overrides = {}) {
  return {
    id: faker.string.uuid(),
    name: faker.company.name(),
    address: faker.location.streetAddress(),
    city: faker.location.city(),
    state: faker.location.state(),
    zipCode: faker.location.zipCode(),
    ...overrides,
  };
}
```

**Usage:**
```typescript
test('User can create property', async () => {
  const user = createTestUser();
  const property = createTestProperty({ ownerId: user.id });

  // Use in test...
});
```

### Database Seeding for E2E Tests

```typescript
// e2e/helpers/seed.ts
export async function seedDatabase(data: {
  users?: User[];
  properties?: Property[];
}) {
  // Clear existing data
  await db('properties').del();
  await db('users').del();

  // Insert test data
  if (data.users) {
    await db('users').insert(data.users);
  }
  if (data.properties) {
    await db('properties').insert(data.properties);
  }
}

// Use in tests
test.beforeEach(async () => {
  await seedDatabase({
    users: [createTestUser({ email: 'test@example.com' })],
    properties: [createTestProperty()],
  });
});
```

---

## Test Organization

### Directory Structure

```
project/
├── backend/
│   ├── src/
│   │   └── models/
│   │       ├── user.rs
│   │       └── user_test.rs       # Unit tests alongside source
│   └── tests/
│       ├── integration/           # Integration tests
│       │   ├── auth_test.rs
│       │   └── bookings_test.rs
│       └── helpers/               # Test utilities
│           ├── setup.rs
│           └── factories.rs
├── frontend/
│   ├── src/
│   │   └── components/
│   │       ├── LoginForm.tsx
│   │       └── LoginForm.test.tsx # Unit tests alongside source
│   └── e2e/                       # E2E tests
│       ├── auth.spec.ts
│       ├── bookings.spec.ts
│       └── helpers/
│           └── seed.ts
└── tests/
    └── load/                      # Load tests
        └── booking-flow.js
```

---

## Running Tests

### Makefile Commands

```makefile
# Unit tests
test-unit: ## Run all unit tests
	@make backend-test-unit
	@make frontend-test-unit

backend-test-unit: ## Run backend unit tests
	@cd backend && cargo test --lib

frontend-test-unit: ## Run frontend unit tests
	@cd frontend && npm test -- --run

# Integration tests
test-integration: ## Run integration tests
	@cd backend && cargo test --test '*'

# E2E tests
test-e2e: ## Run E2E tests
	@cd frontend && npx playwright test

test-e2e-mobile: ## Run E2E tests on mobile
	@cd frontend && npx playwright test --project="Mobile Safari"

# All tests
test: ## Run all tests
	@make test-unit
	@make test-integration
	@make test-e2e

# Coverage
test-coverage: ## Run tests with coverage
	@cd backend && cargo tarpaulin --out Html
	@cd frontend && npm test -- --coverage

# Load tests
load-test: ## Run load tests
	@k6 run tests/load/booking-flow.js
```

### CI Commands

```makefile
ci-quick: ## Fast CI checks (< 3 min)
	@make lint
	@make test-unit

ci: ## Full CI validation (< 10 min)
	@make lint
	@make test-unit
	@make test-integration
	@make test-e2e
	@make test-coverage
```

---

## Continuous Integration

### Local-First CI

**All validation runs locally BEFORE pushing:**

```bash
# Pre-commit hook (automatic)
make pre-commit

# Manual validation
make ci-quick    # Fast (< 3 min)
make ci          # Full (< 10 min)
```

### CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run unit tests
        run: make test-unit

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Start dependencies
        run: docker-compose up -d postgres redis
      - name: Run integration tests
        run: make test-integration

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Start services
        run: docker-compose -f docker-compose.e2e.yml up -d
      - name: Run E2E tests
        run: make test-e2e
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: frontend/playwright-report/
```

---

## Test Performance

### Parallel Execution

**Unit and integration tests:**
```rust
// Rust: Tests run in parallel by default
#[tokio::test]
async fn test_concurrent_execution() {
    // Each test gets isolated state
}
```

```typescript
// Jest: Run tests in parallel
// package.json
{
  "scripts": {
    "test": "jest --maxWorkers=4"
  }
}
```

**E2E tests:**
```typescript
// playwright.config.ts
export default defineConfig({
  fullyParallel: true,  // Run all tests in parallel
  workers: process.env.CI ? 2 : 4,  // 2 workers in CI, 4 locally
});
```

### Speed Targets

- **Unit tests:** < 1 minute total
- **Integration tests:** < 3 minutes total
- **E2E tests:** < 10 minutes total
- **Full CI:** < 15 minutes total

---

## Debugging Tests

### Unit Tests

```bash
# Rust: Run single test with output
cargo test test_name -- --nocapture

# TypeScript: Run single test
npm test -- LoginForm.test.tsx
```

### E2E Tests

```bash
# Run in debug mode (interactive)
make test-e2e-debug

# Or directly
cd frontend && npx playwright test --debug

# Run specific test
npx playwright test auth.spec.ts --debug

# Show test report
npx playwright show-report
```

### Visual Debugging

```typescript
// Add screenshot on failure
test('critical flow', async ({ page }, testInfo) => {
  try {
    // Test steps...
  } catch (error) {
    await page.screenshot({
      path: `test-results/failure-${testInfo.title}.png`,
    });
    throw error;
  }
});
```

---

## Common Testing Anti-Patterns

### ❌ Testing Implementation Details

**BAD:**
```typescript
test('useState is called', () => {
  const spy = jest.spyOn(React, 'useState');
  render(<LoginForm />);
  expect(spy).toHaveBeenCalled();
});
```

**GOOD:**
```typescript
test('form submits with email and password', () => {
  const onSubmit = jest.fn();
  render(<LoginForm onSubmit={onSubmit} />);

  fireEvent.change(screen.getByLabelText('Email'), {
    target: { value: 'user@example.com' },
  });
  fireEvent.click(screen.getByRole('button', { name: 'Login' }));

  expect(onSubmit).toHaveBeenCalledWith({ email: 'user@example.com' });
});
```

### ❌ Not Isolating Tests

**BAD:**
```typescript
let user;
beforeAll(async () => {
  user = await createUser(); // Shared between all tests
});

test('test 1', async () => {
  await updateUser(user.id, { name: 'New Name' }); // Affects other tests!
});
```

**GOOD:**
```typescript
beforeEach(async () => {
  await seedDatabase(); // Fresh state for each test
});

afterEach(async () => {
  await cleanupDatabase(); // Clean up after each test
});
```

### ❌ Hardcoded Waits

**BAD:**
```typescript
await page.click('button');
await page.waitForTimeout(3000); // Flaky!
```

**GOOD:**
```typescript
await page.click('button');
await expect(page.locator('[data-testid="success"]')).toBeVisible();
```

---

## Testing Checklist

### For Every User Story

- [ ] E2E test written BEFORE implementation
- [ ] E2E test covers acceptance criteria
- [ ] Backend unit tests for business logic
- [ ] Backend integration tests for API endpoints
- [ ] Frontend unit tests for components
- [ ] Test coverage ≥ 80%
- [ ] Tests pass locally
- [ ] Tests pass in CI

### For Every PR

- [ ] All tests passing
- [ ] No test skips without justification
- [ ] New tests added for new functionality
- [ ] Coverage not decreased
- [ ] E2E tests updated if user workflows changed

---

## Success Metrics

**Testing is successful when:**

- ✅ All critical user workflows have E2E coverage
- ✅ Code coverage ≥ 80%
- ✅ Test suite runs in < 15 minutes
- ✅ Flakiness rate < 1%
- ✅ Tests catch real bugs before production
- ✅ Developers write tests first (TDD)
- ✅ Tests are trusted (green = safe to deploy)

---

## Related Documents

- [API Testing Requirements](./API_TESTING_REQUIREMENTS.md) - Mandatory API integration testing standards
- [E2E Testing Best Practices](./E2E_TESTING.md) - Detailed E2E testing guide
- [Dev-First Approach](../methodology/DEV_FIRST_APPROACH.md) - Testing infrastructure setup
- [User Story Template](../../templates/user-story/TEMPLATE.md) - E2E test scenarios
- [Makefile Reference](../development/MAKEFILE_REFERENCE.md) - Test commands

---

**Remember: Tests are not optional. Quality is not negotiable.**
