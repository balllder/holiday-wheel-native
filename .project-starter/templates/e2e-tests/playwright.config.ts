import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E Test Configuration
 *
 * MANDATORY REQUIREMENTS:
 * - Minimum 4 workers for concurrent execution
 * - All tests must be isolated (unique test data per test)
 * - No shared sessions or storage state between tests
 *
 * See: https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  // Test directory
  testDir: './e2e',

  // MANDATORY: Enable fully parallel execution
  // All tests run concurrently without dependencies
  fullyParallel: true,

  // Fail build on CI if you accidentally left test.only
  forbidOnly: !!process.env.CI,

  // Retry failed tests on CI only
  retries: process.env.CI ? 2 : 0,

  // MANDATORY: Minimum 4 concurrent workers for parallel execution
  // Each worker runs tests in isolation with unique test data
  workers: process.env.CI ? 4 : 6, // 4 in CI, 6 locally for faster execution

  // Reporter configuration
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['list'], // Show test progress during execution
  ],

  // Shared settings for all projects
  use: {
    // Base URL for navigation
    baseURL: process.env.BASE_URL || 'http://localhost:4200',

    // API endpoint for backend calls (if different from baseURL)
    // extraHTTPHeaders can be used to set API base URL
    extraHTTPHeaders: {
      'Accept': 'application/json',
    },

    // Collect trace on first retry
    trace: 'on-first-retry',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video on failure
    video: 'retain-on-failure',

    // Each worker gets a unique storage state to prevent session conflicts
    // Tests must create their own sessions - no shared authentication
    storageState: undefined,

    // Maximum time per action (click, fill, etc.)
    actionTimeout: 10_000, // 10 seconds

    // Maximum time per navigation
    navigationTimeout: 30_000, // 30 seconds
  },

  // Configure projects for major browsers
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    // Uncomment to test on Firefox
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },

    // Uncomment to test on WebKit (Safari)
    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },

    // Mobile viewports
    // {
    //   name: 'Mobile Chrome',
    //   use: { ...devices['Pixel 5'] },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: { ...devices['iPhone 12'] },
    // },

    // Branded browsers
    // {
    //   name: 'Microsoft Edge',
    //   use: { ...devices['Desktop Edge'], channel: 'msedge' },
    // },
    // {
    //   name: 'Google Chrome',
    //   use: { ...devices['Desktop Chrome'], channel: 'chrome' },
    // },
  ],

  // Start dev server before tests (optional)
  // Uncomment and adjust to your project
  // webServer: {
  //   command: 'npm run dev',
  //   url: 'http://localhost:4200',
  //   reuseExistingServer: !process.env.CI,
  //   timeout: 120_000, // 2 minutes
  // },
});
