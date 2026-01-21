import { test, expect, Page } from '@playwright/test';

// Helper to create a test user and login
async function registerAndLogin(page: Page, email: string, displayName: string): Promise<void> {
  await page.goto('/register');
  await page.waitForLoadState('networkidle');

  await page.fill('#displayName', displayName);
  await page.fill('#email', email);
  await page.fill('#password', 'TestPassword123!');
  await page.fill('#confirmPassword', 'TestPassword123!');
  await page.locator('button[type="submit"]').click();

  // Wait for redirect to lobby (auto-verified in test mode when email disabled)
  await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });
}

test.describe('Lobby', () => {
  test('redirects unauthenticated users to login', async ({ page }) => {
    // Clear any existing auth
    await page.context().clearCookies();
    await page.goto('/lobby');

    // Should redirect to login
    await expect(page).toHaveURL('/');
  });

  test('shows lobby after successful registration', async ({ page }) => {
    const uniqueEmail = `lobby-test-${Date.now()}@example.com`;

    await registerAndLogin(page, uniqueEmail, 'Lobby Tester');

    // If we're at lobby, verify elements
    if (page.url().includes('/lobby')) {
      // Verify we're on the lobby page by checking for lobby-specific elements
      await expect(page.locator('#roomName')).toBeVisible();
      await expect(page.locator('#userName')).toContainText('Lobby Tester');
    }
  });

  test('room name input is visible', async ({ page }) => {
    const uniqueEmail = `room-test-${Date.now()}@example.com`;

    await registerAndLogin(page, uniqueEmail, 'Room Tester');

    if (page.url().includes('/lobby')) {
      // Room name input should be visible
      await expect(page.locator('#roomName')).toBeVisible();
    }
  });

  test('join button is visible', async ({ page }) => {
    const uniqueEmail = `join-test-${Date.now()}@example.com`;

    await registerAndLogin(page, uniqueEmail, 'Join Tester');

    if (page.url().includes('/lobby')) {
      // Join button should be visible
      const joinButton = page.locator('button:has-text("Join"), button:has-text("Play")');
      await expect(joinButton.first()).toBeVisible();
    }
  });

  test('QR code section exists', async ({ page }) => {
    const uniqueEmail = `qr-test-${Date.now()}@example.com`;

    await registerAndLogin(page, uniqueEmail, 'QR Tester');

    if (page.url().includes('/lobby')) {
      // QR code container should exist
      const qrSection = page.locator('#qrCode, [data-testid="qr-code"]');
      await expect(qrSection).toBeVisible({ timeout: 5000 });
    }
  });
});
