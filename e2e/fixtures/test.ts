import { test as base, expect, BrowserContext, Page } from '@playwright/test';
import {
  deleteUserByGithubLogin,
  SESSION_COOKIE_NAME,
} from './auth';
import { closePool } from './db';

/**
 * Mock user configuration for OAuth flow.
 * These values are passed to the mock OAuth server via query params.
 */
export interface MockUser {
  id: number;
  login: string;
  name: string;
  email: string;
}

export interface AuthFixtures {
  /** The mock user configuration used for OAuth */
  mockUser: MockUser;
  /** Page that is already authenticated via mock OAuth flow */
  authenticatedPage: Page;
}

/**
 * Extended test with authentication fixtures using mock OAuth.
 *
 * Usage:
 * ```typescript
 * import { test, expect } from '../fixtures/test';
 *
 * test('authenticated test', async ({ authenticatedPage, mockUser }) => {
 *   await authenticatedPage.goto('/');
 *   await expect(authenticatedPage.getByText(mockUser.login)).toBeVisible();
 * });
 * ```
 */
export const test = base.extend<AuthFixtures>({
  // Generate a unique mock user config for each test
  mockUser: async ({}, use) => {
    // Use random number + timestamp for uniqueness across parallel workers
    const uniqueId = Date.now() * 1000 + Math.floor(Math.random() * 1000);
    const user: MockUser = {
      id: uniqueId,
      login: `testuser_${uniqueId}`,
      name: `Test User ${uniqueId}`,
      email: `test${uniqueId}@example.com`,
    };
    await use(user);
  },

  // Create an authenticated page via mock OAuth flow
  authenticatedPage: async ({ browser, mockUser }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    // First navigate to home to establish a session
    await page.goto('/');

    // Build the auth URL with mock user params
    const authUrl = `/auth/github?mock_user_id=${mockUser.id}&mock_user_login=${encodeURIComponent(mockUser.login)}&mock_user_name=${encodeURIComponent(mockUser.name)}&mock_user_email=${encodeURIComponent(mockUser.email)}`;

    // Navigate to auth endpoint - this will:
    // 1. Redirect to mock OAuth server with mock params
    // 2. Mock server immediately redirects back with code
    // 3. App exchanges code for token and creates user
    // 4. App redirects to home page
    await page.goto(authUrl);

    // Wait for the OAuth flow to complete and redirect back home
    await page.waitForURL('/', { timeout: 10000 });

    // Verify we're logged in by checking for the welcome message
    await expect(page.getByText(`Welcome, ${mockUser.login}!`)).toBeVisible({ timeout: 5000 });

    await use(page);

    // Cleanup: close context and delete the user created by OAuth
    await context.close();
    await deleteUserByGithubLogin(mockUser.login);
  },
});

export { expect };

/**
 * Helper to clear the session cookie (logout simulation).
 */
export async function clearSessionCookie(context: BrowserContext): Promise<void> {
  await context.clearCookies({ name: SESSION_COOKIE_NAME });
}

// Global teardown - close the database pool after all tests
base.afterAll(async () => {
  await closePool();
});
