import { test, expect } from '../fixtures/test';

test.describe('Homepage - Authenticated User', () => {
  test('displays user info when logged in', async ({ authenticatedPage, mockUser }) => {
    // web-app[verify homepage.public]
    await authenticatedPage.goto('/');

    // User's GitHub login name is displayed
    // web-app[verify homepage.auth.welcome]
    await expect(authenticatedPage.getByText(`Welcome, ${mockUser.login}!`)).toBeVisible();

    // User's avatar is displayed (img with the avatar URL)
    const avatar = authenticatedPage.locator('img[alt="Avatar"]');
    // web-app[verify homepage.auth.avatar]
    await expect(avatar).toBeVisible();
  });

  test('shows navigation links for authenticated users', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/');

    // Profile link is visible
    // web-app[verify homepage.auth.profile-link]
    await expect(authenticatedPage.getByRole('link', { name: 'Profile' })).toBeVisible();

    // Battlesnakes link is visible
    // web-app[verify homepage.auth.battlesnakes-link]
    await expect(authenticatedPage.getByRole('link', { name: 'Battlesnakes' })).toBeVisible();

    // Logout link is visible
    // web-app[verify homepage.auth.logout-link]
    await expect(authenticatedPage.getByRole('link', { name: 'Logout' })).toBeVisible();
  });

  test('does not show login link when authenticated', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/');

    // Login link should NOT be visible
    // web-app[verify homepage.auth.no-login-link]
    await expect(authenticatedPage.getByRole('link', { name: 'Login with GitHub' })).not.toBeVisible();
  });
});
