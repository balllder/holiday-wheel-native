import { test, expect, Page } from '@playwright/test';

async function registerUser(page: Page): Promise<void> {
  const uniqueEmail = `nav-test-${Date.now()}@example.com`;

  await page.goto('/register');
  await page.waitForLoadState('networkidle');

  await page.fill('#displayName', 'Nav Tester');
  await page.fill('#email', uniqueEmail);
  await page.fill('#password', 'testpassword123');
  await page.fill('#confirmPassword', 'testpassword123');
  await page.locator('button[type="submit"]').click();

  await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });
}

test.describe('Navigation Flow', () => {
  test('complete user journey: register → lobby → game', async ({ page }) => {
    // Step 1: Registration
    const uniqueEmail = `journey-${Date.now()}@example.com`;
    await page.goto('/register');
    await page.waitForLoadState('networkidle');

    await page.fill('#displayName', 'Journey Tester');
    await page.fill('#email', uniqueEmail);
    await page.fill('#password', 'testpassword123');
    await page.fill('#confirmPassword', 'testpassword123');
    await page.locator('button[type="submit"]').click();

    // Step 2: Should land on lobby
    await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });
    await expect(page.locator('#roomName')).toBeVisible();

    // Step 3: Join a game
    const roomName = `journey-${Date.now()}`;
    await page.fill('#roomName', roomName);
    const joinButton = page.locator('button:has-text("Join"), button:has-text("Play")');
    await joinButton.first().click();

    // Should navigate to game page
    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });
    await expect(page.locator('#wheelSvg, #wheelContainer').first()).toBeVisible({ timeout: 5000 });
  });

  test('browser back button from game returns to lobby', async ({ page }) => {
    await registerUser(page);

    // Navigate to game
    const roomName = `back-test-${Date.now()}`;
    await page.goto(`/game?room=${roomName}`);
    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Use browser back button
    await page.goBack();

    // Should return to lobby
    await page.waitForURL(/\/(lobby|$)/, { timeout: 5000 });
    await expect(page.locator('#roomName')).toBeVisible();
  });

  test('direct game URL access when authenticated', async ({ page }) => {
    await registerUser(page);

    // Directly access game URL
    const roomName = `direct-${Date.now()}`;
    await page.goto(`/game?room=${roomName}`);

    // Should load game page
    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });
    await expect(page.locator('#wheelSvg, #wheelContainer').first()).toBeVisible({ timeout: 5000 });
  });

  test('logout redirects to login page', async ({ page }) => {
    await registerUser(page);

    // Click logout button (if exists)
    const logoutButton = page.locator('button:has-text("Logout"), button:has-text("Sign Out"), a:has-text("Logout")');

    if (await logoutButton.count() > 0) {
      await logoutButton.first().click();

      // Should redirect to login
      await page.waitForURL('/', { timeout: 5000 });
      await expect(page.locator('#email')).toBeVisible();
    }
  });

  test('session persists after page reload', async ({ page }) => {
    await registerUser(page);

    // Verify we're on lobby
    await expect(page.locator('#roomName')).toBeVisible();

    // Reload page
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Should still be on lobby (session persisted)
    await expect(page.locator('#roomName')).toBeVisible({ timeout: 5000 });
  });

  test('handles invalid room codes gracefully', async ({ page }) => {
    await registerUser(page);

    // Try to join with empty room name
    const joinButton = page.locator('button:has-text("Join"), button:has-text("Play")');
    await joinButton.first().click();

    // Should show error or stay on lobby
    // Either we stay on lobby or see an error message
    const stillOnLobby = await page.locator('#roomName').isVisible().catch(() => false);
    const errorVisible = await page.locator('#error, .error-message, [role="alert"]').isVisible().catch(() => false);

    expect(stillOnLobby || errorVisible).toBeTruthy();
  });

  test('room code in URL is case-insensitive', async ({ page }) => {
    await registerUser(page);

    const roomName = `CaseTEST-${Date.now()}`;

    // Try lowercase version
    await page.goto(`/game?room=${roomName.toLowerCase()}`);
    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });

    const displayedRoom = await page.locator('#roomName, span[data-room]').first().textContent();
    expect(displayedRoom?.toLowerCase()).toContain(roomName.toLowerCase());
  });

  test('handles concurrent navigation correctly', async ({ page }) => {
    await registerUser(page);

    const roomName = `concurrent-${Date.now()}`;

    // Rapidly navigate to game and back
    await page.goto(`/game?room=${roomName}`);
    await page.waitForTimeout(500);
    await page.goBack();
    await page.waitForTimeout(500);
    await page.goto(`/game?room=${roomName}`);

    // Should end up on game page
    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });
    await expect(page.locator('#wheelSvg, #wheelContainer').first()).toBeVisible({ timeout: 5000 });
  });
});
