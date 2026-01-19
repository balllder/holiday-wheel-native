import { test, expect, Page } from '@playwright/test';

async function setupAuthenticatedUser(page: Page): Promise<string> {
  const uniqueEmail = `game-test-${Date.now()}@example.com`;

  await page.goto('/register');
  await page.fill('#displayName', 'Game Tester');
  await page.fill('#email', uniqueEmail);
  await page.fill('#password', 'testpassword123');
  await page.fill('#confirmPassword', 'testpassword123');
  await page.click('button[type="submit"]');

  await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });
  return uniqueEmail;
}

test.describe('Game Page', () => {
  test('redirects unauthenticated users to login', async ({ page }) => {
    await page.context().clearCookies();
    await page.goto('/game?room=test');
    await expect(page).toHaveURL('/');
  });

  test('loads game interface with room parameter', async ({ page }) => {
    await setupAuthenticatedUser(page);

    const roomName = `game-${Date.now()}`;

    // If at lobby, join a room
    if (page.url().includes('/lobby')) {
      await page.fill('#roomName', roomName);
      const joinButton = page.locator('button:has-text("Join"), button:has-text("Play")');
      await joinButton.first().click();
    } else {
      // Direct navigation
      await page.goto(`/game?room=${roomName}`);
    }

    // Wait for game page to load
    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Check game elements exist
    await expect(page.locator('#roomName, .room-name')).toContainText(roomName);
  });

  test('displays wheel element', async ({ page }) => {
    await setupAuthenticatedUser(page);

    const roomName = `wheel-test-${Date.now()}`;

    if (page.url().includes('/lobby')) {
      await page.fill('#roomName', roomName);
      const joinButton = page.locator('button:has-text("Join"), button:has-text("Play")');
      await joinButton.first().click();
    } else {
      await page.goto(`/game?room=${roomName}`);
    }

    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Wheel element should be visible
    const wheel = page.locator('#wheel, .wheel, canvas, svg');
    await expect(wheel.first()).toBeVisible({ timeout: 5000 });
  });

  test('displays puzzle board', async ({ page }) => {
    await setupAuthenticatedUser(page);

    const roomName = `puzzle-test-${Date.now()}`;

    if (page.url().includes('/lobby')) {
      await page.fill('#roomName', roomName);
      const joinButton = page.locator('button:has-text("Join"), button:has-text("Play")');
      await joinButton.first().click();
    } else {
      await page.goto(`/game?room=${roomName}`);
    }

    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Puzzle board should exist
    const puzzleBoard = page.locator('#puzzle-board, .puzzle-board, #puzzleBoard');
    await expect(puzzleBoard.first()).toBeVisible({ timeout: 5000 });
  });

  test('displays player area', async ({ page }) => {
    await setupAuthenticatedUser(page);

    const roomName = `player-test-${Date.now()}`;

    if (page.url().includes('/lobby')) {
      await page.fill('#roomName', roomName);
      const joinButton = page.locator('button:has-text("Join"), button:has-text("Play")');
      await joinButton.first().click();
    } else {
      await page.goto(`/game?room=${roomName}`);
    }

    await page.waitForURL(/\/game\?room=/, { timeout: 10000 });

    // Players area should exist
    const players = page.locator('#players, .players, .player-list');
    await expect(players.first()).toBeVisible({ timeout: 5000 });
  });
});
