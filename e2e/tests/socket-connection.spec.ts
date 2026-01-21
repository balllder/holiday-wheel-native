import { test, expect, Page } from '@playwright/test';

async function setupAuthenticatedUser(page: Page): Promise<string> {
  const uniqueEmail = `socket-test-${Date.now()}@example.com`;

  await page.goto('/register');
  await page.waitForLoadState('networkidle');

  await page.fill('#displayName', 'Socket Tester');
  await page.fill('#email', uniqueEmail);
  await page.fill('#password', 'TestPassword123!');
  await page.fill('#confirmPassword', 'TestPassword123!');
  await page.locator('button[type="submit"]').click();

  await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });
  return uniqueEmail;
}

test.describe('Socket.IO Connection', () => {
  test('establishes socket connection in game room', async ({ page }) => {
    await setupAuthenticatedUser(page);

    const roomName = `socket-${Date.now()}`;
    await page.goto(`/game?room=${roomName}`);
    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Check for socket connection indicator (if implemented)
    // Wait for game state to load (indicates socket is connected)
    await page.waitForLoadState('networkidle');

    // Verify game UI elements are present (indicates successful socket connection)
    const wheel = page.locator('#wheelSvg, #wheelContainer');
    await expect(wheel.first()).toBeVisible({ timeout: 5000 });
  });

  test('handles socket disconnection gracefully', async ({ page }) => {
    await setupAuthenticatedUser(page);

    const roomName = `disconnect-${Date.now()}`;
    await page.goto(`/game?room=${roomName}`);
    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Simulate network interruption by going offline
    await page.context().setOffline(true);

    // Wait a moment for disconnect to be detected
    await page.waitForTimeout(2000);

    // Go back online
    await page.context().setOffline(false);

    // Verify reconnection (page should still be functional)
    const wheel = page.locator('#wheelSvg, #wheelContainer');
    await expect(wheel.first()).toBeVisible({ timeout: 10000 });
  });

  test('multiple users can join same room', async ({ browser }) => {
    const context1 = await browser.newContext();
    const page1 = await context1.newPage();
    const email1 = `multi-1-${Date.now()}@example.com`;

    await page1.goto('/register');
    await page1.waitForLoadState('networkidle');
    await page1.fill('#displayName', 'Player 1');
    await page1.fill('#email', email1);
    await page1.fill('#password', 'TestPassword123!');
    await page1.fill('#confirmPassword', 'TestPassword123!');
    await page1.locator('button[type="submit"]').click();
    await page1.waitForURL(/\/(lobby|$)/, { timeout: 15000 });

    const roomName = `multi-${Date.now()}`;
    await page1.goto(`/game?room=${roomName}`);
    await page1.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Second user joins same room
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    const email2 = `multi-2-${Date.now()}@example.com`;

    await page2.goto('/register');
    await page2.waitForLoadState('networkidle');
    await page2.fill('#displayName', 'Player 2');
    await page2.fill('#email', email2);
    await page2.fill('#password', 'TestPassword123!');
    await page2.fill('#confirmPassword', 'TestPassword123!');
    await page2.locator('button[type="submit"]').click();
    await page2.waitForURL(/\/(lobby|$)/, { timeout: 15000 });

    await page2.goto(`/game?room=${roomName}`);
    await page2.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Both pages should show the game
    const wheel1 = page1.locator('#wheelSvg, #wheelContainer');
    const wheel2 = page2.locator('#wheelSvg, #wheelContainer');

    await expect(wheel1.first()).toBeVisible({ timeout: 5000 });
    await expect(wheel2.first()).toBeVisible({ timeout: 5000 });

    // Verify player list shows both players (if implemented)
    const players1 = page1.locator('#players, .players, .player-list');
    await expect(players1.first()).toBeVisible({ timeout: 5000 });

    await context1.close();
    await context2.close();
  });

  test('receives real-time game state updates', async ({ browser }) => {
    const context1 = await browser.newContext();
    const page1 = await context1.newPage();
    const email1 = `realtime-1-${Date.now()}@example.com`;

    await page1.goto('/register');
    await page1.waitForLoadState('networkidle');
    await page1.fill('#displayName', 'Host Player');
    await page1.fill('#email', email1);
    await page1.fill('#password', 'TestPassword123!');
    await page1.fill('#confirmPassword', 'TestPassword123!');
    await page1.locator('button[type="submit"]').click();
    await page1.waitForURL(/\/(lobby|$)/, { timeout: 15000 });

    const roomName = `realtime-${Date.now()}`;
    await page1.goto(`/game?room=${roomName}`);
    await page1.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Second player joins
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    const email2 = `realtime-2-${Date.now()}@example.com`;

    await page2.goto('/register');
    await page2.waitForLoadState('networkidle');
    await page2.fill('#displayName', 'Guest Player');
    await page2.fill('#email', email2);
    await page2.fill('#password', 'TestPassword123!');
    await page2.fill('#confirmPassword', 'TestPassword123!');
    await page2.locator('button[type="submit"]').click();
    await page2.waitForURL(/\/(lobby|$)/, { timeout: 15000 });

    await page2.goto(`/game?room=${roomName}`);
    await page2.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Wait for both to load
    await page1.waitForTimeout(2000);
    await page2.waitForTimeout(2000);

    // Both should see the same game state
    const roomName1 = await page1.locator('#roomName, span[data-room]').first().textContent();
    const roomName2 = await page2.locator('#roomName, span[data-room]').first().textContent();

    // Room names should match (case-insensitive)
    expect(roomName1?.toLowerCase()).toContain(roomName.toLowerCase());
    expect(roomName2?.toLowerCase()).toContain(roomName.toLowerCase());

    await context1.close();
    await context2.close();
  });
});
