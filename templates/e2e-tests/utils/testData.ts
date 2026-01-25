/**
 * Test Data Utilities
 *
 * Helper functions for generating unique test data.
 * CRITICAL: All test data must be unique to support parallel execution.
 */

/**
 * Create a unique email address for testing
 *
 * Format: prefix-timestamp-random@example.com
 *
 * @param prefix - Descriptive prefix (e.g., "login", "register")
 * @returns Unique email address
 */
export function createUniqueEmail(prefix: string): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(7);
  return `${prefix}-${timestamp}-${random}@example.com`;
}

/**
 * Generate a valid password meeting complexity requirements
 *
 * Requirements:
 * - Minimum 12 characters
 * - At least one uppercase letter
 * - At least one lowercase letter
 * - At least one number
 * - At least one special character
 *
 * @returns Valid password
 */
export function generateValidPassword(): string {
  const timestamp = Date.now();
  return `Test@Pass${timestamp}!`;
}

/**
 * Generate a unique username
 *
 * @param prefix - Descriptive prefix
 * @returns Unique username
 */
export function createUniqueUsername(prefix: string): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(7);
  return `${prefix}_${timestamp}_${random}`;
}

/**
 * Generate random string of specified length
 *
 * @param length - Length of string to generate
 * @returns Random alphanumeric string
 */
export function randomString(length: number): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * Generate random number within range
 *
 * @param min - Minimum value (inclusive)
 * @param max - Maximum value (inclusive)
 * @returns Random number
 */
export function randomNumber(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

/**
 * Wait for specified duration (for testing delays)
 *
 * @param ms - Milliseconds to wait
 */
export function wait(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Create unique test data object
 *
 * Example usage:
 * ```typescript
 * const user = createTestUser('testuser');
 * // Returns: { email: "testuser-123456-abc@example.com", password: "Test@Pass123456!" }
 * ```
 */
export function createTestUser(prefix: string) {
  return {
    email: createUniqueEmail(prefix),
    password: generateValidPassword(),
    username: createUniqueUsername(prefix),
  };
}
