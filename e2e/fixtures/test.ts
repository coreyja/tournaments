import { test as base, expect, Page } from '@playwright/test';
import { query } from './db';

// Mock GitHub OAuth server URL (matches playwright.config.ts)
const MOCK_GITHUB_PORT = process.env.MOCK_GITHUB_PORT || '8081';
const MOCK_GITHUB_URL = `http://localhost:${MOCK_GITHUB_PORT}`;

/**
 * Mock user configuration for OAuth flow.
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
  /** Helper to login as a different user on a page */
  loginAsUser: (page: Page, user: MockUser) => Promise<void>;
}

/**
 * Generate a unique mock user for testing.
 */
export function createMockUser(prefix: string = 'testuser'): MockUser {
  const uniqueId = Date.now() * 1000 + Math.floor(Math.random() * 1000);
  return {
    id: uniqueId,
    login: `${prefix}_${uniqueId}`,
    name: `Test User ${uniqueId}`,
    email: `test${uniqueId}@example.com`,
  };
}

/**
 * Delete a user created via mock OAuth by their github_login.
 * Used for cleanup after tests that use the mock OAuth flow.
 */
async function deleteUserByGithubLogin(githubLogin: string): Promise<void> {
  // Delete sessions first (foreign key constraint)
  await query(`
    DELETE FROM sessions
    WHERE user_id IN (
      SELECT user_id FROM users WHERE github_login = $1
    )
  `, [githubLogin]);

  // Delete battlesnakes owned by user
  await query(`
    DELETE FROM battlesnakes
    WHERE user_id IN (
      SELECT user_id FROM users WHERE github_login = $1
    )
  `, [githubLogin]);

  // Delete the user
  await query('DELETE FROM users WHERE github_login = $1', [githubLogin]);
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
    await use(createMockUser('testuser'));
  },

  // Helper to login as a different user on a page
  loginAsUser: async ({}, use) => {
    const usersToCleanup: string[] = [];

    const loginFn = async (page: Page, user: MockUser) => {
      // Track user for cleanup
      usersToCleanup.push(user.login);

      // Set up route handler for this user's OAuth flow
      await page.route('**/auth/github', async (route) => {
        const requestUrl = route.request().url();
        const requestHeaders = route.request().headers();

        const nativeResponse = await fetch(requestUrl, {
          method: 'GET',
          headers: requestHeaders,
          redirect: 'manual',
        });

        const locationHeader = nativeResponse.headers.get('location');
        if (locationHeader && locationHeader.includes('/login/oauth/authorize')) {
          const parsedUrl = new URL(locationHeader);
          const state = parsedUrl.searchParams.get('state');

          if (state) {
            await fetch(`${MOCK_GITHUB_URL}/_admin/set-user-for-state`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                state,
                user: {
                  id: user.id,
                  login: user.login,
                  name: user.name,
                  email: user.email,
                  avatar_url: 'https://example.com/avatar.png',
                },
              }),
            });
          }
        }

        await route.fulfill({
          status: nativeResponse.status,
          headers: Object.fromEntries(nativeResponse.headers.entries()),
          body: await nativeResponse.text(),
        });
      });

      // Navigate to auth endpoint
      await page.goto('/auth/github');
      await page.waitForURL('/', { timeout: 10000 });

      // Unroute after login to avoid conflicts with future logins
      await page.unroute('**/auth/github');
    };

    await use(loginFn);

    // Cleanup all users created via loginAsUser
    for (const login of usersToCleanup) {
      await deleteUserByGithubLogin(login);
    }
  },

  // Create an authenticated page via mock OAuth flow
  authenticatedPage: async ({ browser, mockUser }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    // First navigate to home to establish a session
    await page.goto('/');

    // Set up route handler to intercept the /auth/github request
    // and extract the OAuth state from the redirect response before following it
    await page.route('**/auth/github', async (route) => {
      // Use native fetch with redirect: 'manual' to get the 302/303 response
      const requestUrl = route.request().url();
      const requestHeaders = route.request().headers();

      const nativeResponse = await fetch(requestUrl, {
        method: 'GET',
        headers: requestHeaders,
        redirect: 'manual',
      });

      // Extract state from the redirect Location header
      const locationHeader = nativeResponse.headers.get('location');
      if (locationHeader && locationHeader.includes('/login/oauth/authorize')) {
        const parsedUrl = new URL(locationHeader);
        const state = parsedUrl.searchParams.get('state');

        if (state) {
          // Register our mock user for this OAuth state BEFORE following the redirect
          await fetch(`${MOCK_GITHUB_URL}/_admin/set-user-for-state`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              state,
              user: {
                id: mockUser.id,
                login: mockUser.login,
                name: mockUser.name,
                email: mockUser.email,
                avatar_url: 'https://example.com/avatar.png',
              },
            }),
          });
        }
      }

      // Fulfill with the redirect response - browser will follow it
      await route.fulfill({
        status: nativeResponse.status,
        headers: Object.fromEntries(nativeResponse.headers.entries()),
        body: await nativeResponse.text(),
      });
    });

    // Navigate to auth endpoint - route handler will intercept and register user before redirect
    await page.goto('/auth/github');

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
