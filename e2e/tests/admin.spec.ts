import { test, expect, Page } from '@playwright/test';

async function registerUser(page: Page, email: string, displayName: string): Promise<void> {
  await page.goto('/register');
  await page.waitForLoadState('networkidle');
  await page.fill('#displayName', displayName);
  await page.fill('#email', email);
  await page.fill('#password', 'TestPassword123!');
  await page.fill('#confirmPassword', 'TestPassword123!');
  await page.locator('button[type="submit"]').click();
  await page.waitForURL(/\/(lobby|$)/, { timeout: 15000 });
}

test.describe('Admin Panel', () => {
  test('redirects unauthenticated users to login', async ({ page }) => {
    await page.context().clearCookies();
    await page.goto('/admin');
    await expect(page).toHaveURL('/');
  });

  test('denies access to non-admin users', async ({ page }) => {
    const uniqueEmail = `nonadmin-${Date.now()}@example.com`;

    await registerUser(page, uniqueEmail, 'Regular User');

    // Navigate to admin page
    await page.goto('/admin');

    // Should show access denied message
    const accessDenied = page.locator('#accessDenied, .access-denied');
    await expect(accessDenied).toBeVisible({ timeout: 5000 });
  });

  test('admin page shows access denied with link to lobby', async ({ page }) => {
    const uniqueEmail = `admin-structure-${Date.now()}@example.com`;

    await registerUser(page, uniqueEmail, 'Structure Tester');

    await page.goto('/admin');
    await page.waitForLoadState('networkidle');

    // Regular user should see access denied
    const accessDenied = page.locator('#accessDenied');
    await expect(accessDenied).toBeVisible({ timeout: 10000 });

    // Should have link back to lobby
    const lobbyLink = accessDenied.locator('a[href="/lobby"]');
    await expect(lobbyLink).toBeVisible();
  });

  test('admin button hidden in lobby for regular users', async ({ page }) => {
    const uniqueEmail = `adminbtn-${Date.now()}@example.com`;

    await registerUser(page, uniqueEmail, 'Button Tester');

    if (page.url().includes('/lobby')) {
      // Admin button should be hidden or have hidden class
      const adminBtn = page.locator('#adminBtn');
      const isHidden = await adminBtn.evaluate((el) => {
        return el.classList.contains('hidden') ||
               window.getComputedStyle(el).display === 'none' ||
               window.getComputedStyle(el).visibility === 'hidden';
      }).catch(() => true);

      expect(isHidden).toBeTruthy();
    }
  });
});
