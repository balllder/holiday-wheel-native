# End-to-End Testing Best Practices

## What is an E2E Test?

**Definition:** An end-to-end test validates a complete user workflow from start to finish, testing the system as a whole from the user's perspective. It runs against the fully integrated application (frontend + backend + database + external services).

---

## MANDATORY E2E Test Requirements

**An E2E test MUST:**

1. **Use the UI exposed to end users**
   - Interact through the actual browser UI (buttons, forms, links)
   - Navigate through real pages and components
   - No direct API calls or backend shortcuts

2. **Cover the complete lifecycle steps for end users**
   - Test ENTIRE user journeys from start to finish
   - Include all intermediate steps the user goes through
   - Example: Sign in → perform action → sign out → sign in again (comprehensive)
   - NOT: Just CRUD operations or single-step flows (incomplete)

3. **NOT use mocking unless explicitly reviewed and accepted**
   - Use real services and dependencies by default
   - **Only mock when:**
     - ✅ Cannot deploy the dependency internally
     - ✅ Not mocking will incur extra costs (paid third-party APIs)
   - All other cases: Use real implementations

**An E2E test IS NOT:**

- ❌ API-only tests (testing endpoints directly)
- ❌ Tests limited to a subset of user flow
- ❌ Single CRUD operation tests (create, read, update, or delete alone)
- ❌ Tests that skip UI and interact with backend directly

**Example of Comprehensive E2E:**
```typescript
// ✅ COMPREHENSIVE E2E: Complete user lifecycle
test('User complete authentication lifecycle', async ({ page }) => {
  // 1. Sign up
  await page.goto('/register');
  await page.fill('[name="email"]', 'user@example.com');
  await page.fill('[name="password"]', 'SecurePass123!');
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/dashboard');

  // 2. Sign out
  await page.click('[data-testid="user-menu"]');
  await page.click('[data-testid="logout"]');
  await expect(page).toHaveURL('/login');

  // 3. Sign in again
  await page.fill('[name="email"]', 'user@example.com');
  await page.fill('[name="password"]', 'SecurePass123!');
  await page.click('button[type="submit"]');
  await expect(page).toHaveURL('/dashboard');
});

// ❌ NOT E2E: Incomplete flow
test('User can create a booking', async ({ page }) => {
  await apiClient.post('/bookings', { ... }); // Direct API call - NOT E2E
  await page.goto('/bookings/123'); // Starting mid-flow - NOT E2E
});
```

---

## Core Principles

### 1. User Perspective
- Tests what a user actually does, not internal implementation
- Uses real UI interactions (clicks, form fills, navigation)
- Validates what users see and experience
- Follows real user journeys from beginning to end

### 2. Full Stack Integration
Tests exercise the entire system:
- ✅ Frontend (React/UI components)
- ✅ Backend API
- ✅ Database (real data persistence)
- ✅ Authentication flows
- ✅ Real dependencies (unless exception criteria met)
- ✅ Visual/UI rendering

### 3. Business Value Focus
- Tests user stories and acceptance criteria from start to finish
- Validates complete business workflows, not technical implementations
- Example: "User registers → verifies email → logs in → updates profile → logs out → logs in again" (E2E)
- NOT: "Registration API returns 201 status" (Integration test)
- NOT: "User can create a booking" without sign-in/sign-out (Incomplete E2E)

---

## Test Isolation (CRITICAL)

**Every E2E test MUST be completely isolated.**

### Why?
- Enables concurrent test execution (faster CI)
- Prevents test interdependencies and flakiness
- Each test can run independently in any order

### How?
```typescript
test.beforeEach(async ({ page }) => {
  // 1. Seed database with fresh test data
  await seedDatabase({
    users: [testUser],
    properties: [testProperty],
  });

  // 2. Clear browser state
  await page.context().clearCookies();
  await page.context().clearPermissions();

  // 3. Start from known state
  await page.goto('/');
});

test.afterEach(async () => {
  // Clean up test data
  await cleanupDatabase();
});
```

### Database Isolation
- Each test gets its own database snapshot OR
- Each test uses isolated test data with unique identifiers
- Use database transactions (rollback after test) OR
- Use containerized databases (one per test suite)

---

## Test Data Strategy

### Real-Looking Data (Not Minimal Test Data)

**❌ BAD:**
```typescript
const user = { email: 'a@b.c', name: 'A' };
```

**✅ GOOD:**
```typescript
const user = {
  email: 'sarah.johnson@example.com',
  name: 'Sarah Johnson',
  phone: '+1 (555) 123-4567',
  address: '123 Main Street, San Francisco, CA 94102',
};
```

### Why Real-Looking Data?
- Exposes edge cases (long names, special characters, international formats)
- Tests realistic UI rendering (text overflow, wrapping, truncation)
- Validates data formatting and display logic
- Better represents production data

### Test Data Factories

```typescript
// e2e/factories/user.factory.ts
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
```

---

## External Services Strategy

### Default: Use Real Services

**E2E tests MUST use real services and dependencies by default.**

### When to Mock (EXCEPTIONS ONLY)

**You may ONLY mock when ONE of these conditions is true:**

1. ✅ **Cannot deploy the dependency internally**
   - Third-party service without local/test environment
   - Service requires special infrastructure we cannot replicate
   - Example: Payment processors (Stripe, PayPal) without sandbox

2. ✅ **Not mocking will incur extra costs**
   - Paid API calls (SMS providers, email services)
   - Usage-based pricing that adds up in CI
   - Example: Twilio SMS ($0.01 per message × 1000 test runs = $10)

**All mocking decisions MUST be explicitly reviewed and documented.**

### Decision Tree:

```
Is it a service we develop?
├─ YES → ✅ Use REAL service (docker-compose)
└─ NO → Is it a third-party service?
    ├─ Can we deploy it internally? (e.g., PostgreSQL, Redis)
    │  └─ YES → ✅ Use REAL service
    └─ NO → Does it cost money per call?
        ├─ YES → ✅ Mock (document reason)
        └─ NO → Does it have a free sandbox?
            ├─ YES → ✅ Use real sandbox
            └─ NO → ✅ Mock (document reason)
```

### Example: Use Real Internal Service (PREFERRED)

```yaml
# docker-compose.e2e.yml
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: test_db
      POSTGRES_USER: test
      POSTGRES_PASSWORD: test

  redis:
    image: redis:7

  auth-service:
    image: auth-service:latest
    ports:
      - "8081:8081"
    depends_on:
      - postgres
      - redis

  main-app:
    image: main-app:latest
    ports:
      - "8080:8080"
    environment:
      AUTH_SERVICE_URL: http://auth-service:8081
    depends_on:
      - auth-service
```

### Example: Mock Third-Party Service (EXCEPTION)

**IMPORTANT: Document WHY you're mocking!**

```typescript
// e2e/mocks/stripe.mock.ts
/**
 * MOCKING JUSTIFICATION:
 * - Service: Stripe Payment API
 * - Reason: Cannot deploy internally + costs $0.01 per test charge
 * - Approved by: [Name] on [Date]
 * - Review ticket: PROJ-123
 */
test.beforeEach(async ({ page }) => {
  await page.route('**/api.stripe.com/**', (route) => {
    route.fulfill({
      status: 200,
      body: JSON.stringify({
        id: 'ch_mock_123',
        status: 'succeeded',
      }),
    });
  });
});
```

### Example: Use Real Third-Party Sandbox (PREFERRED)

```typescript
// e2e/config/stripe.config.ts
/**
 * Using Stripe's FREE test mode
 * - No mocking needed
 * - Real API integration
 * - Free for testing
 */
const stripeConfig = {
  apiKey: process.env.STRIPE_TEST_KEY, // Test mode key (free)
  apiBase: 'https://api.stripe.com', // Real API
};
```

---

## Mobile and Responsive Testing

### 80/20 Coverage Strategy

**Desktop Tests (80%)**: All critical workflows
- Run on standard desktop viewport (1920x1080)
- Focus on functionality, not responsive behavior

**Mobile Tests (20%)**: Mobile-critical features only
- Responsive navigation (hamburger menus)
- Touch interactions (swipe, tap, pinch-to-zoom)
- Mobile-first features (location, camera, QR codes)
- Forms on mobile (keyboard, input focus, autocomplete)

### Playwright Configuration

```typescript
// playwright.config.ts
export default defineConfig({
  projects: [
    {
      name: 'Desktop Chrome',
      use: {
        viewport: { width: 1920, height: 1080 },
      },
    },
    {
      name: 'Mobile Safari',
      use: {
        ...devices['iPhone 13'],
        hasTouch: true,
      },
    },
    {
      name: 'Mobile Chrome',
      use: {
        ...devices['Pixel 5'],
        hasTouch: true,
      },
    },
  ],
});
```

### Test Organization

```
e2e/
├── critical-flows/           # Run on BOTH desktop and mobile
│   ├── authentication.spec.ts
│   ├── checkout.spec.ts
│   └── booking.spec.ts
├── desktop/                  # Desktop-only tests
│   ├── admin-panel.spec.ts
│   └── complex-data-tables.spec.ts
└── mobile/                   # Mobile-specific tests
    ├── mobile-navigation.spec.ts
    ├── touch-interactions.spec.ts
    └── responsive-layouts.spec.ts
```

### Example: Critical Flow on Multiple Devices

```typescript
// e2e/critical-flows/authentication.spec.ts
import { test, devices } from '@playwright/test';

const testDevices = [
  { name: 'Desktop', viewport: { width: 1920, height: 1080 } },
  { name: 'Mobile', ...devices['iPhone 13'] },
];

testDevices.forEach(({ name, ...device }) => {
  test.describe(`Authentication - ${name}`, () => {
    test.use(device);

    test('User can register and login', async ({ page }) => {
      await page.goto('/register');
      await page.fill('[name="email"]', 'sarah@example.com');
      await page.fill('[name="password"]', 'SecurePass123!');
      await page.click('button[type="submit"]');

      await expect(page).toHaveURL('/dashboard');
      await expect(page.locator('h1')).toContainText('Welcome');
    });
  });
});
```

### Mobile-Specific Assertions

```typescript
// e2e/mobile/mobile-navigation.spec.ts
test('Mobile hamburger menu works', async ({ page }) => {
  const isMobile = page.viewportSize()!.width < 768;

  if (isMobile) {
    // Mobile: menu hidden, hamburger visible
    await expect(page.locator('nav')).toBeHidden();
    await page.click('[data-testid="hamburger-menu"]');
    await expect(page.locator('nav')).toBeVisible();
  } else {
    // Desktop: menu always visible
    await expect(page.locator('nav')).toBeVisible();
  }
});
```

---

## Performance Testing

### Functional Performance (Every E2E Test)

**Assert on load times for critical user journeys:**

```typescript
test('Page loads within acceptable time', async ({ page }) => {
  const startTime = Date.now();

  await page.goto('/dashboard');
  await page.waitForLoadState('networkidle');

  const loadTime = Date.now() - startTime;
  expect(loadTime).toBeLessThan(3000); // 3 seconds max
});
```

### Playwright Performance APIs

```typescript
test('Measure page performance', async ({ page }) => {
  await page.goto('/dashboard');

  const performanceMetrics = await page.evaluate(() => {
    const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
    return {
      domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
      loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
      firstPaint: performance.getEntriesByType('paint')[0]?.startTime,
    };
  });

  expect(performanceMetrics.domContentLoaded).toBeLessThan(1500); // 1.5s
  expect(performanceMetrics.loadComplete).toBeLessThan(3000); // 3s
});
```

### Load Testing (Separate Test Suite)

**Use tools like k6, Artillery, or Playwright with multiple workers:**

```typescript
// e2e/load/concurrent-users.spec.ts
import { test } from '@playwright/test';

test.describe.configure({ mode: 'parallel' });

// Simulate 50 concurrent users
for (let i = 0; i < 50; i++) {
  test(`User ${i} can browse and book`, async ({ page }) => {
    await page.goto('/properties');
    await page.click('[data-testid="property-1"]');
    await page.click('[data-testid="book-now"]');
    // Complete booking flow
  });
}
```

### Load Testing with k6 (Recommended)

```javascript
// load-tests/booking-flow.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 50 },   // Ramp up to 50 users
    { duration: '3m', target: 50 },   // Stay at 50 users
    { duration: '1m', target: 100 },  // Ramp up to 100 users
    { duration: '3m', target: 100 },  // Stay at 100 users
    { duration: '1m', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests under 500ms
    http_req_failed: ['rate<0.01'],   // Less than 1% errors
  },
};

export default function () {
  const res = http.get('http://localhost:8080/api/v1/properties');
  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
  sleep(1);
}
```

**Run load tests:**
```bash
make load-test          # Run k6 load tests
k6 run load-tests/booking-flow.js
```

---

## Visual/UI Testing

### UI is Part of Functionality

**Visual checks are part of E2E tests, not separate.**

### 1. Visual Regression Testing

```typescript
// e2e/visual/landing-page.spec.ts
test('Landing page renders correctly', async ({ page }) => {
  await page.goto('/');
  await page.waitForLoadState('networkidle');

  // Take screenshot and compare to baseline
  await expect(page).toHaveScreenshot('landing-page.png', {
    maxDiffPixels: 100, // Allow small differences
  });
});
```

### 2. Component Visual States

```typescript
test('Button states render correctly', async ({ page }) => {
  await page.goto('/components');

  // Default state
  await expect(page.locator('button')).toHaveScreenshot('button-default.png');

  // Hover state
  await page.hover('button');
  await expect(page.locator('button')).toHaveScreenshot('button-hover.png');

  // Disabled state
  await page.locator('button').evaluate((el) => el.setAttribute('disabled', 'true'));
  await expect(page.locator('button')).toHaveScreenshot('button-disabled.png');
});
```

### 3. Responsive Visual Testing

```typescript
test('Responsive layout renders correctly', async ({ page }) => {
  await page.goto('/dashboard');

  // Desktop
  await page.setViewportSize({ width: 1920, height: 1080 });
  await expect(page).toHaveScreenshot('dashboard-desktop.png');

  // Tablet
  await page.setViewportSize({ width: 768, height: 1024 });
  await expect(page).toHaveScreenshot('dashboard-tablet.png');

  // Mobile
  await page.setViewportSize({ width: 375, height: 812 });
  await expect(page).toHaveScreenshot('dashboard-mobile.png');
});
```

### 4. Accessibility Visual Checks

```typescript
import AxeBuilder from '@axe-core/playwright';

test('Page has no accessibility violations', async ({ page }) => {
  await page.goto('/dashboard');

  const accessibilityScanResults = await new AxeBuilder({ page }).analyze();

  expect(accessibilityScanResults.violations).toEqual([]);
});
```

---

## What E2E Tests Should Cover

### ✅ DO Test:

1. **Complete User Lifecycles** (MANDATORY)
   - FULL workflows from start to finish
   - Include ALL steps a user goes through
   - ✅ Example: Register → Verify email → Login → Update profile → Logout → Login again
   - ❌ NOT: Just "user can login" (incomplete lifecycle)

2. **Critical Happy Paths** (80% of user value)
   - User registration, verification, login, logout
   - Core business workflows (booking, checkout, payments)
   - COMPLETE CRUD cycles (create → read → update → delete in ONE test)

3. **User Workflows Through the UI** (MANDATORY)
   - MUST interact through browser UI (clicks, forms, navigation)
   - Cross-page navigation and state persistence
   - Multi-step forms and wizards
   - NO direct API calls or backend shortcuts

4. **Data Persistence Across Interactions**
   - Data saved to database correctly
   - Data retrieved and displayed correctly
   - Data updates reflected across the app
   - State maintained across sign-out/sign-in cycles

5. **Error States Users Encounter**
   - Form validation errors shown in UI
   - Network errors (API failures) displayed to user
   - Authentication errors (expired session) with redirect
   - User-facing error messages and recovery flows

6. **Authentication Flows (Complete)**
   - Login → use app → logout → login again
   - Password reset → receive email → set new password → login
   - Multi-factor authentication setup → logout → login with MFA
   - Passkey/WebAuthn registration → logout → login with passkey

7. **UI Rendering and Visual States**
   - Page layouts render correctly in browser
   - Components in different states (loading, error, empty, success)
   - Responsive layouts on different viewports

### ❌ DO NOT Test:

1. **Internal APIs Directly** → Integration tests (not E2E)
2. **Incomplete User Flows** → Must cover full lifecycle
3. **Individual Functions** → Unit tests
4. **Every Possible Edge Case** → Unit tests (test pyramid)
5. **Implementation Details** (React state, component internals)
6. **Non-User-Facing Logic** (internal calculations, helpers)
7. **Backend-Only Operations** → Integration tests
8. **Single CRUD Operations** → Must test complete CRUD cycle

---

## Test-Driven Development (TDD) with E2E Tests

### Write E2E Tests BEFORE Implementation

**Process:**
1. User story defined
2. **E2E test written** (fails - RED)
3. Feature implemented (test passes - GREEN)
4. Refactor (test still passes)

### Example: E2E-First Development

```typescript
// Step 1: Write E2E test FIRST (from user story acceptance criteria)
// e2e/user-registration.spec.ts
test('User can register with email and password', async ({ page }) => {
  await page.goto('/register');

  await page.fill('[name="email"]', 'sarah@example.com');
  await page.fill('[name="password"]', 'SecurePass123!');
  await page.fill('[name="confirmPassword"]', 'SecurePass123!');
  await page.click('button[type="submit"]');

  // Assertions based on acceptance criteria
  await expect(page).toHaveURL('/dashboard');
  await expect(page.locator('[data-testid="welcome-message"]')).toContainText('Welcome, Sarah!');
  await expect(page.locator('[data-testid="email-verification-banner"]')).toBeVisible();
});

// Test will FAIL because feature doesn't exist yet (RED)

// Step 2: Implement the feature (GREEN)
// - Create /register route
// - Build registration form
// - Implement backend API
// - Add validation
// - Test passes!

// Step 3: Refactor (test still passes)
// - Clean up code
// - Extract components
// - Optimize performance
// - Test still passes!
```

---

## Best Practices

### 1. Use Data Test IDs

**❌ BAD: Brittle selectors**
```typescript
await page.click('.btn.btn-primary.submit-button');
```

**✅ GOOD: Stable selectors**
```typescript
await page.click('[data-testid="submit-button"]');
```

### 2. Wait for Stable State

```typescript
// Wait for network requests to complete
await page.waitForLoadState('networkidle');

// Wait for specific elements
await page.waitForSelector('[data-testid="dashboard"]');

// Wait for API response
await page.waitForResponse('**/api/v1/user');
```

### 3. Avoid Hardcoded Waits

**❌ BAD:**
```typescript
await page.click('button');
await page.waitForTimeout(3000); // Flaky!
```

**✅ GOOD:**
```typescript
await page.click('button');
await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
```

### 4. Group Related Tests

```typescript
test.describe('User Authentication', () => {
  test('User can register', async ({ page }) => { /* ... */ });
  test('User can login', async ({ page }) => { /* ... */ });
  test('User can logout', async ({ page }) => { /* ... */ });
});
```

### 5. Use Page Object Model (POM) for Complex Flows

```typescript
// e2e/pages/registration.page.ts
export class RegistrationPage {
  constructor(private page: Page) {}

  async goto() {
    await this.page.goto('/register');
  }

  async fillEmail(email: string) {
    await this.page.fill('[name="email"]', email);
  }

  async fillPassword(password: string) {
    await this.page.fill('[name="password"]', password);
  }

  async submit() {
    await this.page.click('button[type="submit"]');
  }

  async expectSuccessfulRegistration() {
    await expect(this.page).toHaveURL('/dashboard');
    await expect(this.page.locator('[data-testid="welcome-message"]')).toBeVisible();
  }
}

// Use in test
test('User can register', async ({ page }) => {
  const registrationPage = new RegistrationPage(page);

  await registrationPage.goto();
  await registrationPage.fillEmail('sarah@example.com');
  await registrationPage.fillPassword('SecurePass123!');
  await registrationPage.submit();
  await registrationPage.expectSuccessfulRegistration();
});
```

---

## Makefile Integration

```makefile
# Run all E2E tests
test-e2e: ## Run all E2E tests
	@echo "Running E2E tests..."
	@cd frontend && npx playwright test

# Run E2E tests on specific device
test-e2e-mobile: ## Run E2E tests on mobile devices only
	@cd frontend && npx playwright test --project="Mobile Safari" --project="Mobile Chrome"

test-e2e-desktop: ## Run E2E tests on desktop only
	@cd frontend && npx playwright test --project="Desktop Chrome"

# Run visual regression tests
test-visual: ## Run visual regression tests
	@cd frontend && npx playwright test e2e/visual/

# Run load tests
load-test: ## Run load tests with k6
	@k6 run load-tests/booking-flow.js

# Debug E2E tests
test-e2e-debug: ## Run E2E tests in debug mode
	@cd frontend && npx playwright test --debug

# Show E2E test report
test-e2e-report: ## Show E2E test report
	@cd frontend && npx playwright show-report
```

---

## CI/CD Integration

```yaml
# .github/workflows/e2e-tests.yml
name: E2E Tests

on:
  pull_request:
  push:
    branches: [main]

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start services
        run: docker-compose -f docker-compose.e2e.yml up -d

      - name: Wait for services
        run: |
          timeout 60 sh -c 'until curl -f http://localhost:8080/health; do sleep 1; done'

      - name: Run E2E tests
        run: make test-e2e

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: frontend/playwright-report/

      - name: Upload screenshots
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: screenshots
          path: frontend/test-results/
```

---

## Troubleshooting

### Flaky Tests

**Problem:** Tests pass sometimes, fail other times

**Solutions:**
- Remove hardcoded waits (`waitForTimeout`)
- Wait for stable state (`waitForLoadState('networkidle')`)
- Ensure test isolation (independent database state)
- Check for race conditions (API responses arriving out of order)

### Slow Tests

**Problem:** E2E tests take too long

**Solutions:**
- Run tests in parallel (`fullyParallel: true` in Playwright config)
- Reduce unnecessary navigation (test multiple things on same page)
- Use faster test data seeding
- Skip non-critical animations (`page.emulateMedia({ reducedMotion: 'reduce' })`)

### Test Data Conflicts

**Problem:** Tests interfere with each other

**Solutions:**
- Ensure complete test isolation (own database snapshot per test)
- Use unique test data identifiers (UUIDs, timestamps)
- Clean up data in `afterEach` hooks

---

## Success Metrics

**E2E test suite is successful when:**

- ✅ All critical user workflows have E2E coverage
- ✅ Tests run in < 10 minutes (parallel execution)
- ✅ Flakiness rate < 1% (tests pass consistently)
- ✅ Tests catch real bugs before production
- ✅ Developers trust the test suite (green = safe to deploy)
- ✅ E2E tests are written BEFORE feature implementation (TDD)
- ✅ New features cannot be merged without E2E tests

---

## Summary

**E2E tests validate COMPLETE user lifecycles from the user's perspective through the browser UI, exercising the full stack (frontend + backend + database) with real services, isolated test data, and visual checks included.**

**MANDATORY Requirements:**
1. **Use the UI exposed to end users** (no direct API calls)
2. **Cover complete lifecycle steps** (full workflows from start to finish)
3. **Use real services by default** (only mock when cannot deploy internally OR incurs extra costs)
4. **Document all mocking decisions** (explicit review and approval required)

**Key Principles:**
1. Test from user perspective through browser UI (not internal APIs)
2. Test COMPLETE user lifecycles (not partial flows)
3. Complete test isolation for concurrency
4. Real-looking test data (not minimal data)
5. Use real services (mock only with documented justification)
6. Selective mobile coverage (80/20 rule)
7. Include performance assertions
8. Visual/UI checks are part of E2E tests
9. Write E2E tests BEFORE implementation (TDD)

**Related Documents:**
- [Testing Guide](../TESTING_GUIDE.md) - Overall testing strategy
- [User Story Template](../../templates/user-story/TEMPLATE.md) - E2E test scenarios
- [Dev-First Approach](../methodology/DEV_FIRST_APPROACH.md) - Testing infrastructure setup
