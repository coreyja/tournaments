import { test as base, expect, BrowserContext, Page } from '@playwright/test';
import {
  createTestUser,
  createTestSession,
  deleteTestUser,
  SESSION_COOKIE_NAME,
  TestUser,
  TestSession,
} from './auth';
import { closePool } from './db';

export interface AuthFixtures {
  /** The authenticated test user */
  testUser: TestUser;
  /** The test session linked to testUser */
  testSession: TestSession;
  /** Page that is already authenticated as testUser */
  authenticatedPage: Page;
}

/**
 * Extended test with authentication fixtures.
 *
 * Usage:
 * ```typescript
 * import { test, expect } from '../fixtures/test';
 *
 * test('authenticated test', async ({ authenticatedPage, testUser }) => {
 *   await authenticatedPage.goto('/');
 *   await expect(authenticatedPage.getByText(testUser.github_login)).toBeVisible();
 * });
 * ```
 */
export const test = base.extend<AuthFixtures>({
  // Create a test user - standalone fixture
  testUser: async ({}, use) => {
    const user = await createTestUser();
    await use(user);
    // Cleanup after test
    await deleteTestUser(user.user_id);
  },

  // Create a session linked to the test user
  testSession: async ({ testUser }, use) => {
    const session = await createTestSession(testUser.user_id);
    await use(session);
    // Session is deleted when testUser is cleaned up (cascade)
  },

  // Create an authenticated page using the test login endpoint
  authenticatedPage: async ({ browser, testUser, testSession }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    // Navigate to the test login endpoint - this will set the encrypted session cookie
    // and redirect to the home page. The browser will follow the redirect and apply cookies.
    // This endpoint is only available when E2E_TEST_MODE is set.
    await page.goto(`/test/auth/login/${testSession.session_id}`, {
      waitUntil: 'networkidle',
    });

    // Verify we ended up at a valid page (should have redirected to /)
    const url = page.url();
    if (url.includes('/test/auth/login/')) {
      throw new Error(
        `Failed to authenticate test user - still at login URL. ` +
        `Make sure E2E_TEST_MODE=1 is set when running the server`
      );
    }

    await use(page);

    // Cleanup
    await context.close();
  },
});

export { expect };

/**
 * Helper to authenticate a page using the test login endpoint.
 * Useful when you need to authenticate mid-test.
 */
export async function authenticateAs(
  page: Page,
  sessionId: string
): Promise<void> {
  const response = await page.request.post(`/test/auth/login/${sessionId}`);
  if (!response.ok()) {
    throw new Error(`Failed to authenticate: ${response.status()}`);
  }
}

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
