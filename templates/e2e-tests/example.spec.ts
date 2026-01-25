/**
 * Example E2E Test Suite
 *
 * Demonstrates best practices for end-to-end testing:
 * - Complete isolation (each test has unique data)
 * - API + UI testing patterns
 * - Proper assertions and timeouts
 * - Clear test structure
 *
 * CRITICAL: Tests MUST be isolated to support parallel execution.
 * Each test creates its own data and doesn't depend on other tests.
 */

import { test, expect } from '@playwright/test';

/**
 * Test Data Helpers
 *
 * Generate unique test data for each test run.
 * This prevents conflicts when tests run in parallel.
 */
function createUniqueEmail(prefix: string): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(7);
  return `${prefix}-${timestamp}-${random}@example.com`;
}

function generateValidPassword(): string {
  return `Test@Pass${Date.now()}`;
}

//=============================================================================
// Health Check Tests
//=============================================================================

test.describe('Health Checks', () => {
  test('should return healthy status from liveness endpoint', async ({ request }) => {
    // Call backend API directly
    const response = await request.get('http://localhost:3000/health');

    // Assert response is successful
    expect(response.status()).toBe(200);

    // Parse JSON response
    const health = await response.json();

    // Assert response structure
    expect(health).toHaveProperty('status', 'healthy');
    expect(health).toHaveProperty('service');
    expect(health).toHaveProperty('version');
  });

  test('should return ready status from readiness endpoint', async ({ request }) => {
    const response = await request.get('http://localhost:3000/health/ready');

    // Assert successful response
    expect(response.status()).toBe(200);

    const health = await response.json();

    // Assert response structure
    expect(health).toHaveProperty('status', 'ready');
    expect(health).toHaveProperty('checks');
    expect(health.checks).toHaveProperty('database', 'ok');
  });
});

//=============================================================================
// API Tests (Direct Backend Calls)
//=============================================================================

test.describe('API Tests', () => {
  test('should list items via API', async ({ request }) => {
    // Make API request to list items
    const response = await request.get('http://localhost:3000/api/items');

    // Assert successful response
    expect(response.status()).toBe(200);

    // Parse JSON
    const items = await response.json();

    // Assert response is an array
    expect(Array.isArray(items)).toBe(true);
  });

  test('should create and retrieve item via API', async ({ request }) => {
    const itemName = `Test Item ${Date.now()}`;

    // Create item
    const createResponse = await request.post('http://localhost:3000/api/items', {
      data: { name: itemName }
    });

    expect(createResponse.status()).toBe(201);

    const created = await createResponse.json();
    expect(created).toHaveProperty('id');
    expect(created.name).toBe(itemName);

    // Retrieve the created item
    const getResponse = await request.get(`http://localhost:3000/api/items/${created.id}`);

    expect(getResponse.status()).toBe(200);

    const retrieved = await getResponse.json();
    expect(retrieved.id).toBe(created.id);
    expect(retrieved.name).toBe(itemName);
  });

  test('should update item via API', async ({ request }) => {
    // Create item first
    const createResponse = await request.post('http://localhost:3000/api/items', {
      data: { name: 'Original Name' }
    });
    const created = await createResponse.json();

    // Update the item
    const updatedName = `Updated ${Date.now()}`;
    const updateResponse = await request.put(`http://localhost:3000/api/items/${created.id}`, {
      data: { name: updatedName }
    });

    expect(updateResponse.status()).toBe(200);

    const updated = await updateResponse.json();
    expect(updated.name).toBe(updatedName);
  });

  test('should delete item via API', async ({ request }) => {
    // Create item first
    const createResponse = await request.post('http://localhost:3000/api/items', {
      data: { name: 'To Be Deleted' }
    });
    const created = await createResponse.json();

    // Delete the item
    const deleteResponse = await request.delete(`http://localhost:3000/api/items/${created.id}`);

    expect(deleteResponse.status()).toBe(204);

    // Verify item is deleted
    const getResponse = await request.get(`http://localhost:3000/api/items/${created.id}`);
    expect(getResponse.status()).toBe(404);
  });

  test('should return 400 for invalid item creation', async ({ request }) => {
    // Try to create item without required field
    const response = await request.post('http://localhost:3000/api/items', {
      data: {}  // Missing 'name' field
    });

    expect(response.status()).toBe(400);

    const error = await response.json();
    expect(error).toHaveProperty('error');
  });
});

//=============================================================================
// UI Tests (Browser Interaction)
//=============================================================================

test.describe('Frontend UI', () => {
  test('should load homepage', async ({ page }) => {
    // Navigate to app
    await page.goto('/');

    // Assert page title
    await expect(page).toHaveTitle(/My App/);

    // Assert heading is visible
    const heading = page.getByRole('heading', { name: /welcome/i });
    await expect(heading).toBeVisible();
  });

  test('should navigate to items page', async ({ page }) => {
    await page.goto('/');

    // Click navigation link
    await page.getByRole('link', { name: /items/i }).click();

    // Assert navigation happened
    await expect(page).toHaveURL(/\/items/);

    // Assert items list is visible
    await expect(page.getByRole('heading', { name: /items/i })).toBeVisible();
  });

  test('should display items list', async ({ page }) => {
    await page.goto('/items');

    // Wait for items to load
    await page.waitForSelector('[data-testid="items-list"]', { timeout: 5000 });

    // Assert list exists
    const list = page.locator('[data-testid="items-list"]');
    await expect(list).toBeVisible();
  });

  test('should create new item via UI', async ({ page }) => {
    await page.goto('/items');

    // Click create button
    await page.getByRole('button', { name: /create item/i }).click();

    // Fill form
    const itemName = `UI Test Item ${Date.now()}`;
    await page.fill('input[name="name"]', itemName);

    // Submit form
    await page.getByRole('button', { name: /save|create/i }).click();

    // Assert item appears in list
    await expect(page.getByText(itemName)).toBeVisible({ timeout: 5000 });
  });

  test('should show item details', async ({ page }) => {
    // Create item via API first (faster setup)
    const itemName = `Details Test ${Date.now()}`;
    await page.request.post('http://localhost:3000/api/items', {
      data: { name: itemName }
    });

    // Navigate to items page
    await page.goto('/items');

    // Click on the item
    await page.getByText(itemName).click();

    // Assert details page
    await expect(page).toHaveURL(/\/items\/\d+/);
    await expect(page.getByRole('heading', { name: itemName })).toBeVisible();
  });

  test('should edit item via UI', async ({ page }) => {
    // Create item via API
    const response = await page.request.post('http://localhost:3000/api/items', {
      data: { name: 'Original Name' }
    });
    const item = await response.json();

    // Navigate to item details
    await page.goto(`/items/${item.id}`);

    // Click edit button
    await page.getByRole('button', { name: /edit/i }).click();

    // Update name
    const newName = `Edited ${Date.now()}`;
    await page.fill('input[name="name"]', newName);

    // Save changes
    await page.getByRole('button', { name: /save/i }).click();

    // Assert updated name is displayed
    await expect(page.getByRole('heading', { name: newName })).toBeVisible({ timeout: 5000 });
  });

  test('should delete item via UI', async ({ page }) => {
    // Create item via API
    const response = await page.request.post('http://localhost:3000/api/items', {
      data: { name: 'To Delete' }
    });
    const item = await response.json();

    // Navigate to item details
    await page.goto(`/items/${item.id}`);

    // Click delete button
    await page.getByRole('button', { name: /delete/i }).click();

    // Confirm deletion (if confirmation dialog exists)
    await page.getByRole('button', { name: /confirm|yes|delete/i }).click();

    // Assert redirected to items list
    await expect(page).toHaveURL('/items', { timeout: 5000 });

    // Assert item no longer in list
    await expect(page.getByText('To Delete')).not.toBeVisible();
  });

  test('should show error for invalid form submission', async ({ page }) => {
    await page.goto('/items');

    // Click create button
    await page.getByRole('button', { name: /create item/i }).click();

    // Submit without filling required field
    await page.getByRole('button', { name: /save|create/i }).click();

    // Assert error message is shown
    await expect(
      page.getByText(/required|cannot be empty|please enter/i)
    ).toBeVisible({ timeout: 3000 });
  });
});

//=============================================================================
// Authentication Tests (Example)
//=============================================================================

test.describe('Authentication', () => {
  test('should show login page', async ({ page }) => {
    await page.goto('/login');

    // Assert login form is visible
    await expect(page.getByRole('heading', { name: /login|sign in/i })).toBeVisible();
    await expect(page.locator('input[name="email"]')).toBeVisible();
    await expect(page.locator('input[name="password"]')).toBeVisible();
  });

  test('should successfully login with valid credentials', async ({ page }) => {
    // In real app, pre-create user via API
    const email = createUniqueEmail('login');
    const password = generateValidPassword();

    // Create user (example - adjust to your API)
    await page.request.post('http://localhost:3000/api/auth/register', {
      data: { email, password }
    });

    // Navigate to login
    await page.goto('/login');

    // Fill credentials
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', password);

    // Submit
    await page.getByRole('button', { name: /login|sign in/i }).click();

    // Assert successful login (redirected to dashboard)
    await expect(page).toHaveURL(/\/dashboard|\/home/, { timeout: 10000 });
  });

  test('should show error for invalid credentials', async ({ page }) => {
    await page.goto('/login');

    // Fill invalid credentials
    await page.fill('input[name="email"]', 'nonexistent@example.com');
    await page.fill('input[name="password"]', 'WrongPassword123!');

    // Submit
    await page.getByRole('button', { name: /login|sign in/i }).click();

    // Assert error message
    await expect(
      page.getByText(/invalid credentials|wrong password|incorrect/i)
    ).toBeVisible({ timeout: 5000 });
  });
});

//=============================================================================
// Performance Tests (Example)
//=============================================================================

test.describe('Performance', () => {
  test('homepage should load within 3 seconds', async ({ page }) => {
    const startTime = Date.now();

    await page.goto('/');

    // Wait for page to be fully loaded
    await page.waitForLoadState('networkidle');

    const loadTime = Date.now() - startTime;

    // Assert load time
    expect(loadTime).toBeLessThan(3000);
  });

  test('API response should be fast', async ({ request }) => {
    const startTime = Date.now();

    await request.get('http://localhost:3000/api/items');

    const responseTime = Date.now() - startTime;

    // Assert response time < 500ms
    expect(responseTime).toBeLessThan(500);
  });
});
