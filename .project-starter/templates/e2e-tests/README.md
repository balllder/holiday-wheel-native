# E2E Testing with Playwright

Comprehensive end-to-end testing patterns using Playwright for browser automation and API testing.

## Overview

E2E tests verify your application works correctly from the user's perspective, testing:
- **UI interactions** - Button clicks, form submissions, navigation
- **API endpoints** - Direct backend testing
- **Complete workflows** - Multi-step user journeys
- **Cross-browser compatibility** - Chrome, Firefox, Safari
- **Performance** - Load times, response times

## Files

| File | Purpose |
|------|---------|
| `example.spec.ts` | Comprehensive E2E test examples |
| `playwright.config.ts` | Playwright configuration |
| `utils/testData.ts` | Test data generation utilities |
| `package.json.example` | npm dependencies and scripts |
| `README.md` | This guide |

---

## Quick Start

### 1. Install Dependencies

```bash
# Copy package.json
cp templates/e2e-tests/package.json.example frontend/package.json

# Install Playwright
cd frontend
npm install
npx playwright install --with-deps
```

### 2. Copy Test Files

```bash
# Copy configuration
cp templates/e2e-tests/playwright.config.ts frontend/playwright.config.ts

# Copy example tests
mkdir -p frontend/e2e
cp templates/e2e-tests/example.spec.ts frontend/e2e/example.spec.ts

# Copy utilities
mkdir -p frontend/e2e/utils
cp templates/e2e-tests/utils/testData.ts frontend/e2e/utils/testData.ts
```

### 3. Update Configuration

Edit `playwright.config.ts`:

```typescript
export default defineConfig({
  testDir: './e2e',

  use: {
    // Update to your frontend URL
    baseURL: 'http://localhost:4200',
  },

  // Optional: Start dev server before tests
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:4200',
    reuseExistingServer: !process.env.CI,
  },
});
```

### 4. Run Tests

```bash
# Run all tests
npm test

# Run with UI mode (recommended for development)
npm run test:ui

# Run in headed mode (see browser)
npm run test:headed

# Debug mode (step through tests)
npm run test:debug
```

---

## Test Structure

### Basic Test Pattern

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test('should do something', async ({ page }) => {
    // 1. Setup (navigate, create data)
    await page.goto('/');

    // 2. Action (user interaction)
    await page.click('button');

    // 3. Assert (verify result)
    await expect(page.locator('text=Success')).toBeVisible();
  });
});
```

### API Testing Pattern

```typescript
test('should fetch data from API', async ({ request }) => {
  // Make API request
  const response = await request.get('http://localhost:3000/api/items');

  // Assert status code
  expect(response.status()).toBe(200);

  // Assert response data
  const data = await response.json();
  expect(Array.isArray(data)).toBe(true);
});
```

### UI Testing Pattern

```typescript
test('should submit form', async ({ page }) => {
  await page.goto('/form');

  // Fill form fields
  await page.fill('input[name="name"]', 'John Doe');
  await page.fill('input[name="email"]', 'john@example.com');

  // Submit form
  await page.click('button[type="submit"]');

  // Assert success
  await expect(page).toHaveURL('/success');
  await expect(page.locator('text=Form submitted')).toBeVisible();
});
```

---

## Test Isolation (CRITICAL)

**MANDATORY**: Tests MUST be isolated to support parallel execution.

### ✅ CORRECT: Isolated Tests

```typescript
test('test 1', async ({ page }) => {
  // Create unique data for this test
  const email = `test1-${Date.now()}@example.com`;

  // Test uses only its own data
  await page.request.post('/api/users', { data: { email } });
  // ... rest of test
});

test('test 2', async ({ page }) => {
  // Create different unique data
  const email = `test2-${Date.now()}@example.com`;

  // This test doesn't depend on test 1
  await page.request.post('/api/users', { data: { email } });
  // ... rest of test
});
```

### ❌ WRONG: Shared State

```typescript
// ❌ BAD: Tests depend on each other
test('create user', async ({ page }) => {
  await page.request.post('/api/users', {
    data: { email: 'test@example.com' } // Hardcoded email
  });
});

test('login user', async ({ page }) => {
  // Depends on previous test creating the user
  await page.fill('input[name="email"]', 'test@example.com');
  // ... WILL FAIL when tests run in parallel
});
```

**Why isolation matters:**
- Tests run in parallel (4-6 workers)
- Tests may run in any order
- Shared data causes race conditions
- One test's failure shouldn't affect others

---

## Test Data Generation

Use utilities from `utils/testData.ts`:

```typescript
import {
  createUniqueEmail,
  generateValidPassword,
  createTestUser,
} from './utils/testData';

test('user registration', async ({ page }) => {
  // Generate unique test data
  const user = createTestUser('register');
  // user.email: "register-1234567890-abc@example.com"
  // user.password: "Test@Pass1234567890!"

  await page.fill('input[name="email"]', user.email);
  await page.fill('input[name="password"]', user.password);
  // ...
});
```

**Benefits:**
- Unique data every time
- No conflicts between tests
- Parallel execution works
- Tests can run multiple times

---

## Locator Strategies

### Best to Worst Locator Priority

1. **Role-based (best)**
   ```typescript
   page.getByRole('button', { name: /submit/i })
   page.getByRole('heading', { name: 'Welcome' })
   ```

2. **Test IDs (good)**
   ```typescript
   page.locator('[data-testid="submit-button"]')
   ```

3. **Labels (good for forms)**
   ```typescript
   page.getByLabel('Email address')
   ```

4. **Placeholder (acceptable)**
   ```typescript
   page.getByPlaceholder('Enter your email')
   ```

5. **Text content (okay)**
   ```typescript
   page.getByText('Click here')
   ```

6. **CSS selectors (avoid if possible)**
   ```typescript
   page.locator('.submit-btn')  // Fragile - CSS may change
   ```

7. **XPath (last resort)**
   ```typescript
   page.locator('//button[@class="submit"]')  // Very fragile
   ```

---

## Assertions

### Common Assertions

```typescript
// Visibility
await expect(page.locator('text=Welcome')).toBeVisible();
await expect(page.locator('text=Hidden')).not.toBeVisible();

// URL
await expect(page).toHaveURL('/dashboard');
await expect(page).toHaveURL(/\/items\/\d+/);

// Title
await expect(page).toHaveTitle('My App');

// Text content
await expect(page.locator('h1')).toHaveText('Welcome');
await expect(page.locator('h1')).toContainText('Wel');

// Attributes
await expect(page.locator('button')).toBeDisabled();
await expect(page.locator('button')).toBeEnabled();
await expect(page.locator('input')).toHaveAttribute('type', 'email');

// Count
await expect(page.locator('.item')).toHaveCount(5);

// API responses
expect(response.status()).toBe(200);
expect(data).toHaveProperty('id');
expect(Array.isArray(data)).toBe(true);
```

---

## Makefile Integration

Add E2E test commands to your Makefile:

```makefile
.PHONY: e2e e2e-ui e2e-headed e2e-debug e2e-report

e2e: ## Run E2E tests
	@cd frontend && npm run test

e2e-ui: ## Run E2E tests with UI mode
	@cd frontend && npm run test:ui

e2e-headed: ## Run E2E tests in headed mode
	@cd frontend && npm run test:headed

e2e-debug: ## Debug E2E tests
	@cd frontend && npm run test:debug

e2e-report: ## Show E2E test report
	@cd frontend && npm run test:report

e2e-codegen: ## Generate E2E test code
	@cd frontend && npm run test:codegen
```

Usage:
```bash
make e2e          # Run all tests
make e2e-ui       # Interactive UI mode
make e2e-report   # View test report
```

---

## Best Practices

### ✅ DO

1. **Use unique test data**
   ```typescript
   const email = createUniqueEmail('test');  // Unique every time
   ```

2. **Wait for elements properly**
   ```typescript
   await expect(page.locator('text=Success')).toBeVisible();
   // Don't use arbitrary waits: await page.waitForTimeout(1000)
   ```

3. **Test user workflows, not implementation**
   ```typescript
   // ✅ GOOD: Test what user sees
   await page.click('text=Submit');

   // ❌ BAD: Test implementation details
   await page.click('#submit-btn-id-123');
   ```

4. **Use descriptive test names**
   ```typescript
   test('should display error when email is invalid', ...)
   // Not: test('test 1', ...)
   ```

5. **Group related tests**
   ```typescript
   test.describe('User Registration', () => {
     test('should register with valid data', ...)
     test('should reject invalid email', ...)
   });
   ```

### ❌ DON'T

1. **Don't use hardcoded test data**
   ```typescript
   // ❌ BAD
   const email = 'test@example.com';

   // ✅ GOOD
   const email = createUniqueEmail('test');
   ```

2. **Don't use arbitrary waits**
   ```typescript
   // ❌ BAD
   await page.waitForTimeout(3000);

   // ✅ GOOD
   await expect(page.locator('text=Loaded')).toBeVisible();
   ```

3. **Don't test implementation details**
   ```typescript
   // ❌ BAD: Testing React component state
   const state = await page.evaluate(() => window.reactState);

   // ✅ GOOD: Testing visible UI
   await expect(page.locator('text=Success')).toBeVisible();
   ```

4. **Don't make tests dependent**
   ```typescript
   // ❌ BAD: Test 2 depends on test 1
   test('create user', ...)
   test('login user', ...)  // Assumes user from test 1 exists

   // ✅ GOOD: Each test is independent
   test('login user', async ({ page }) => {
     await createUser();  // Create in this test
     // ... login
   });
   ```

---

## Debugging Tests

### Visual Debugging

```bash
# UI mode (best for debugging)
npm run test:ui

# Headed mode (see browser)
npm run test:headed

# Debug mode (step through)
npm run test:debug
```

### Add Debugging Code

```typescript
test('debug example', async ({ page }) => {
  // Pause execution
  await page.pause();

  // Take screenshot
  await page.screenshot({ path: 'debug.png' });

  // Print page content
  console.log(await page.content());

  // Print element text
  console.log(await page.locator('h1').textContent());
});
```

### View Test Reports

```bash
# Generate and view HTML report
npm run test:report
```

---

## CI Integration

### GitHub Actions Example

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright browsers
        run: npx playwright install --with-deps

      - name: Start services
        run: docker-compose up -d

      - name: Run E2E tests
        run: npm test

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/
```

---

## Advanced Patterns

### Setup and Teardown

```typescript
test.describe('User Management', () => {
  // Runs before each test
  test.beforeEach(async ({ page }) => {
    await page.goto('/admin');
  });

  // Runs after each test
  test.afterEach(async ({ page }) => {
    // Cleanup (if needed)
  });

  test('test 1', async ({ page }) => {
    // Test already at /admin
  });
});
```

### Fixtures (Reusable Setup)

```typescript
// test-fixtures.ts
import { test as base } from '@playwright/test';

type Fixtures = {
  authenticatedPage: Page;
};

export const test = base.extend<Fixtures>({
  authenticatedPage: async ({ page }, use) => {
    // Login before test
    await page.goto('/login');
    await page.fill('input[name="email"]', 'admin@example.com');
    await page.fill('input[name="password"]', 'password');
    await page.click('button[type="submit"]');
    await page.waitForURL('/dashboard');

    // Use the authenticated page
    await use(page);
  },
});

// In test file
import { test } from './test-fixtures';

test('admin can see users', async ({ authenticatedPage }) => {
  // Already logged in!
  await authenticatedPage.goto('/users');
});
```

### Mocking API Responses

```typescript
test('should handle API error', async ({ page }) => {
  // Mock API to return error
  await page.route('**/api/items', route => {
    route.fulfill({
      status: 500,
      body: JSON.stringify({ error: 'Server error' }),
    });
  });

  await page.goto('/items');

  // Assert error is displayed
  await expect(page.locator('text=Error loading items')).toBeVisible();
});
```

---

## Troubleshooting

### Tests are flaky

**Problem:** Tests sometimes pass, sometimes fail

**Solutions:**
1. **Use proper waits**
   ```typescript
   await expect(element).toBeVisible();  // Not: await page.waitForTimeout(1000)
   ```

2. **Ensure test isolation**
   - Each test creates its own data
   - No shared state between tests

3. **Increase timeouts if needed**
   ```typescript
   await expect(element).toBeVisible({ timeout: 10000 });
   ```

---

### Element not found

**Problem:** `locator.click: Error: Element not found`

**Solutions:**
1. **Wait for element**
   ```typescript
   await expect(page.locator('button')).toBeVisible();
   await page.click('button');
   ```

2. **Use better locator**
   ```typescript
   // Instead of: page.click('.btn')
   page.getByRole('button', { name: 'Submit' })
   ```

---

### Tests too slow

**Problem:** Tests take too long

**Solutions:**
1. **Use API for setup**
   ```typescript
   // Instead of: Fill registration form
   // Use: Direct API call to create user
   await page.request.post('/api/users', { data: user });
   ```

2. **Run tests in parallel**
   - Already enabled in config (6 workers locally)
   - Ensure tests are isolated

3. **Skip unnecessary steps**
   - Test one path thoroughly
   - Other tests can use shortcuts

---

## References

- [Playwright Documentation](https://playwright.dev/)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [Testing Library Guiding Principles](https://testing-library.com/docs/guiding-principles/)
- [Auth-Service E2E Tests](https://github.com/brefwiz/auth-service) - Production example

---

**Last Updated:** 2026-01-24
**Extracted from:** auth-service E2E test suite
