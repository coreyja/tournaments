import { test, expect } from '../fixtures/test';

/**
 * Authentication via Mock OAuth
 *
 * [verify auth.oauth.success.redirect]
 * [verify auth.protected.extraction]
 * [verify homepage.auth.welcome]
 */
test.describe('Authentication via Mock OAuth', () => {
  /**
   * [verify auth.oauth.success.redirect]
   * [verify homepage.auth.welcome]
   */
  test('authenticatedPage shows logged in user on homepage', async ({ authenticatedPage, mockUser }) => {
    // authenticatedPage starts at home page after OAuth redirect
    await authenticatedPage.goto('/');

    // Should show the welcome message with user's github login
    await expect(authenticatedPage.getByText(`Welcome, ${mockUser.login}!`)).toBeVisible();
  });

  /**
   * [verify auth.protected.extraction]
   * [verify profile.route]
   * [verify profile.auth_required]
   */
  test('authenticatedPage can access protected routes', async ({ authenticatedPage }) => {
    // Profile page requires authentication (at /me)
    await authenticatedPage.goto('/me');

    // Should load profile page instead of redirecting
    await expect(authenticatedPage).toHaveURL('/me');
    await expect(authenticatedPage.getByRole('heading', { name: 'My Profile' })).toBeVisible();
  });

  /**
   * [verify auth.user.github_id]
   * [verify auth.user.github_login]
   * [verify auth.user.name]
   * [verify auth.user.email]
   */
  test('mockUser has expected properties', async ({ mockUser }) => {
    // MockUser should have the expected shape
    expect(mockUser.id).toBeGreaterThan(0);
    expect(mockUser.login).toMatch(/^testuser_\d+$/);
    expect(mockUser.name).toMatch(/^Test User \d+$/);
    expect(mockUser.email).toMatch(/^test\d+@example\.com$/);
  });
});
