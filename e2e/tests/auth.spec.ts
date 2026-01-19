import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test.beforeEach(async ({ page }) => {
    // Clear storage before each test
    await page.context().clearCookies();
  });

  test('login page loads correctly', async ({ page }) => {
    await page.goto('/');

    // Check page title and key elements
    await expect(page).toHaveTitle(/Holiday Wheel/);
    await expect(page.locator('h1')).toContainText('Holiday Wheel');

    // Check form elements exist
    await expect(page.locator('#email')).toBeVisible();
    await expect(page.locator('#password')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('shows error for invalid credentials', async ({ page }) => {
    await page.goto('/');

    await page.fill('#email', 'invalid@example.com');
    await page.fill('#password', 'wrongpassword123');
    await page.click('button[type="submit"]');

    // Wait for error message
    const error = page.locator('#error');
    await expect(error).toBeVisible({ timeout: 5000 });
  });

  test('register page accessible from login', async ({ page }) => {
    await page.goto('/');

    // Wait for the page to fully load
    await page.waitForLoadState('networkidle');

    const registerLink = page.locator('a[href="/register"]');
    await expect(registerLink).toBeVisible();
    await registerLink.click();

    await expect(page).toHaveURL(/\/register/);
    // The page h1 contains the app name, verify we're on the register page by checking for form fields
    await expect(page.locator('#displayName')).toBeVisible();
  });

  test('registration form has required fields', async ({ page }) => {
    await page.goto('/register');

    // Check required fields exist
    await expect(page.locator('#displayName')).toBeVisible();
    await expect(page.locator('#email')).toBeVisible();
    await expect(page.locator('#password')).toBeVisible();
    await expect(page.locator('#confirmPassword')).toBeVisible();
  });

  test('registration validates password mismatch', async ({ page }) => {
    await page.goto('/register');

    await page.fill('#displayName', 'Test User');
    await page.fill('#email', 'test@example.com');
    await page.fill('#password', 'password123');
    await page.fill('#confirmPassword', 'different123');
    await page.click('button[type="submit"]');

    // Should show password mismatch error
    const error = page.locator('#error');
    await expect(error).toBeVisible({ timeout: 5000 });
    await expect(error).toContainText(/password/i);
  });

  test('successful registration redirects to lobby', async ({ page }) => {
    const uniqueEmail = `test-${Date.now()}@example.com`;

    await page.goto('/register');
    await page.waitForLoadState('networkidle');

    await page.fill('#displayName', 'E2E Test User');
    await page.fill('#email', uniqueEmail);
    await page.fill('#password', 'testpassword123');
    await page.fill('#confirmPassword', 'testpassword123');

    // Wait for button to be ready and click
    const submitBtn = page.locator('button[type="submit"]');
    await expect(submitBtn).toBeVisible();
    await submitBtn.click();

    // Should redirect to lobby after registration (auto-verified in test mode)
    await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });
  });
});
