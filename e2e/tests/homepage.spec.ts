import { test, expect } from '../fixtures/test';

/**
 * Homepage - Authenticated User
 *
 * [verify homepage.auth.welcome]
 * [verify homepage.auth.avatar]
 * [verify homepage.auth.profile_link]
 * [verify homepage.auth.battlesnakes_link]
 * [verify homepage.auth.logout_link]
 * [verify homepage.auth.no_login_link]
 */
test.describe('Homepage - Authenticated User', () => {
  /**
   * [verify homepage.auth.welcome]
   * [verify homepage.auth.avatar]
   */
  test('displays user info when logged in', async ({ authenticatedPage, mockUser }) => {
    await authenticatedPage.goto('/');

    // User's GitHub login name is displayed
    await expect(authenticatedPage.getByText(`Welcome, ${mockUser.login}!`)).toBeVisible();

    // User's avatar is displayed (img with the avatar URL)
    const avatar = authenticatedPage.locator('img[alt="Avatar"]');
    await expect(avatar).toBeVisible();
  });

  /**
   * [verify homepage.auth.profile_link]
   * [verify homepage.auth.battlesnakes_link]
   * [verify homepage.auth.logout_link]
   */
  test('shows navigation links for authenticated users', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/');

    // Profile link is visible
    await expect(authenticatedPage.getByRole('link', { name: 'Profile' })).toBeVisible();

    // Battlesnakes link is visible
    await expect(authenticatedPage.getByRole('link', { name: 'Battlesnakes' })).toBeVisible();

    // Logout link is visible
    await expect(authenticatedPage.getByRole('link', { name: 'Logout' })).toBeVisible();
  });

  /**
   * [verify homepage.auth.no_login_link]
   */
  test('does not show login link when authenticated', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/');

    // Login link should NOT be visible
    await expect(authenticatedPage.getByRole('link', { name: 'Login with GitHub' })).not.toBeVisible();
  });
});
