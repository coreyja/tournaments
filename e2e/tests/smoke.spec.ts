import { test, expect } from '../fixtures/test';

/**
 * Smoke Tests
 *
 * [verify homepage.route]
 * [verify homepage.public]
 * [verify homepage.unauth.message]
 * [verify homepage.unauth.login_link]
 */
test.describe('Smoke Tests', () => {
  test('homepage loads successfully', async ({ page }) => {
    await page.goto('/');

    // Verify the page loads with expected content
    await expect(page.getByRole('heading', { name: 'Hello, world!' })).toBeVisible();
    await expect(page.getByText('Welcome to the Tournaments application!')).toBeVisible();
  });

  test('shows login link for unauthenticated users', async ({ page }) => {
    await page.goto('/');

    await expect(page.getByText('You are not logged in.')).toBeVisible();
    await expect(page.getByRole('link', { name: 'Login with GitHub' })).toBeVisible();
  });
});
