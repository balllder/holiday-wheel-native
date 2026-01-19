import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { open: 'never' }],
    ['list']
  ],
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:5000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  // Start backend server before tests
  webServer: {
    command: 'cd ../apps/backend-rust && cargo run --release',
    url: 'http://localhost:5000/health',
    reuseExistingServer: !process.env.CI,
    timeout: 180 * 1000, // Rust compilation can be slow
    env: {
      PORT: '5000',
      DB_PATH: './test-puzzles.db',
      EMAIL_ENABLED: 'false',
      HOST_CODE: 'testhost',
      RUST_LOG: 'warn',
      JWT_SECRET: 'test-secret-for-e2e',
    },
  },
});
