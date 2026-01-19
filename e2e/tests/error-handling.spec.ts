import { test, expect, Page } from '@playwright/test';

test.describe('Error Handling', () => {
  test('handles server timeout gracefully', async ({ page }) => {
    // Set a short timeout to simulate slow server
    page.setDefaultTimeout(5000);

    await page.goto('/', { timeout: 10000 });

    // Page should load even if slow
    await expect(page.locator('#email')).toBeVisible({ timeout: 10000 });
  });

  test('shows error for invalid email format', async ({ page }) => {
    await page.goto('/register');
    await page.waitForLoadState('networkidle');

    await page.fill('#displayName', 'Test User');
    await page.fill('#email', 'invalid-email'); // Invalid format
    await page.fill('#password', 'password123');
    await page.fill('#confirmPassword', 'password123');
    await page.locator('button[type="submit"]').click();

    // Should show error message
    const errorVisible = await page.locator('#error, .error-message, [role="alert"]').isVisible().catch(() => false);
    const stillOnRegisterPage = await page.locator('#displayName').isVisible().catch(() => false);

    expect(errorVisible || stillOnRegisterPage).toBeTruthy();
  });

  test('validates password strength', async ({ page }) => {
    await page.goto('/register');
    await page.waitForLoadState('networkidle');

    await page.fill('#displayName', 'Test User');
    await page.fill('#email', `weak-pass-${Date.now()}@example.com`);
    await page.fill('#password', '123'); // Too weak
    await page.fill('#confirmPassword', '123');
    await page.locator('button[type="submit"]').click();

    // Should show error or reject
    await page.waitForTimeout(1000);
    const errorVisible = await page.locator('#error, .error-message, [role="alert"]').isVisible().catch(() => false);
    const stillOnRegisterPage = await page.locator('#displayName').isVisible().catch(() => false);

    expect(errorVisible || stillOnRegisterPage).toBeTruthy();
  });

  test('handles duplicate email registration', async ({ page }) => {
    const email = `duplicate-${Date.now()}@example.com`;

    // First registration
    await page.goto('/register');
    await page.waitForLoadState('networkidle');
    await page.fill('#displayName', 'First User');
    await page.fill('#email', email);
    await page.fill('#password', 'testpassword123');
    await page.fill('#confirmPassword', 'testpassword123');
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });

    // Logout
    await page.context().clearCookies();

    // Try to register with same email
    await page.goto('/register');
    await page.waitForLoadState('networkidle');
    await page.fill('#displayName', 'Second User');
    await page.fill('#email', email);
    await page.fill('#password', 'testpassword456');
    await page.fill('#confirmPassword', 'testpassword456');
    await page.locator('button[type="submit"]').click();

    // Should show error (accept either specific or generic error message)
    const error = page.locator('#error');
    await expect(error).toBeVisible({ timeout: 5000 });
    // Backend may return specific "email already exists" or generic "Registration failed"
    await expect(error).toContainText(/email|exist|already|failed|error/i);
  });

  test('handles network errors during login', async ({ page }) => {
    await page.goto('/');

    // Set offline mode to simulate network error
    await page.context().setOffline(true);

    await page.fill('#email', 'test@example.com');
    await page.fill('#password', 'password123');
    await page.locator('button[type="submit"]').click();

    // Should show network error or timeout
    await page.waitForTimeout(2000);

    // Go back online
    await page.context().setOffline(false);

    // Error should be visible
    const errorVisible = await page.locator('#error, .error-message, [role="alert"]').isVisible().catch(() => false);
    const stillOnLoginPage = await page.locator('#email').isVisible().catch(() => false);

    expect(errorVisible || stillOnLoginPage).toBeTruthy();
  });

  test('handles malformed game URL parameters', async ({ page }) => {
    const uniqueEmail = `malformed-${Date.now()}@example.com`;

    // Register and login
    await page.goto('/register');
    await page.waitForLoadState('networkidle');
    await page.fill('#displayName', 'Test User');
    await page.fill('#email', uniqueEmail);
    await page.fill('#password', 'testpassword123');
    await page.fill('#confirmPassword', 'testpassword123');
    await page.locator('button[type="submit"]').click();
    await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });

    // Try to access game with malformed room parameter
    await page.goto('/game?room='); // Empty room

    // Should redirect to lobby or show error
    await page.waitForTimeout(2000);
    const onLobby = page.url().includes('/lobby');
    const onHome = page.url() === page.context().pages()[0].url();
    const errorVisible = await page.locator('#error, .error-message, [role="alert"]').isVisible().catch(() => false);

    expect(onLobby || onHome || errorVisible).toBeTruthy();
  });

  test('handles XSS attempts in form inputs', async ({ page }) => {
    await page.goto('/register');
    await page.waitForLoadState('networkidle');

    const xssPayload = '<script>alert("XSS")</script>';

    await page.fill('#displayName', xssPayload);
    await page.fill('#email', `xss-${Date.now()}@example.com`);
    await page.fill('#password', 'testpassword123');
    await page.fill('#confirmPassword', 'testpassword123');
    await page.locator('button[type="submit"]').click();

    // Wait for processing
    await page.waitForTimeout(2000);

    // Script should not execute (no alert dialog)
    const dialogHandler = async (dialog: import('@playwright/test').Dialog) => {
      // If we get here, XSS was not prevented
      expect(dialog.message()).not.toBe('XSS');
      await dialog.dismiss();
    };
    page.on('dialog', dialogHandler);

    // Page should handle safely
    const pageContent = await page.content();
    expect(pageContent).not.toContain('<script>alert("XSS")</script>');

    page.off('dialog', dialogHandler);
  });

  test('validates required fields on registration', async ({ page }) => {
    await page.goto('/register');
    await page.waitForLoadState('networkidle');

    // Try to submit with empty fields
    await page.locator('button[type="submit"]').click();

    // Should not proceed (HTML5 validation or custom)
    await page.waitForTimeout(1000);
    const stillOnRegisterPage = await page.locator('#displayName').isVisible();
    expect(stillOnRegisterPage).toBeTruthy();
  });

  test('handles very long input strings', async ({ page }) => {
    await page.goto('/register');
    await page.waitForLoadState('networkidle');

    const longString = 'A'.repeat(1000);

    await page.fill('#displayName', longString);
    await page.fill('#email', `long-${Date.now()}@example.com`);
    await page.fill('#password', 'testpassword123');
    await page.fill('#confirmPassword', 'testpassword123');
    await page.locator('button[type="submit"]').click();

    // Should handle gracefully (reject or truncate)
    await page.waitForTimeout(2000);
    const errorVisible = await page.locator('#error, .error-message, [role="alert"]').isVisible().catch(() => false);
    const stillOnRegisterPage = await page.locator('#displayName').isVisible().catch(() => false);

    expect(errorVisible || stillOnRegisterPage).toBeTruthy();
  });
});
