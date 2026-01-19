import { test, expect } from '@playwright/test';

test.describe('Health Check', () => {
  test('health endpoint returns OK', async ({ request }) => {
    const response = await request.get('/health');

    expect(response.ok()).toBeTruthy();

    const body = await response.json();
    expect(body.status).toBe('ok');
  });
});
