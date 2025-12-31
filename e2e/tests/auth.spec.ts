import { test, expect } from '../fixtures/test';

/**
 * Authentication via Mock OAuth
 *
 * web-app[verify auth.oauth.success.redirect]
 * web-app[verify auth.protected.extraction]
 * web-app[verify homepage.auth.welcome]
 */
test.describe('Authentication via Mock OAuth', () => {
  /**
   * web-app[verify auth.oauth.success.redirect]
   * web-app[verify homepage.auth.welcome]
   */
  test('authenticatedPage shows logged in user on homepage', async ({ authenticatedPage, mockUser }) => {
    // authenticatedPage starts at home page after OAuth redirect
    await authenticatedPage.goto('/');

    // Should show the welcome message with user's github login
    await expect(authenticatedPage.getByText(`Welcome, ${mockUser.login}!`)).toBeVisible();
  });

  /**
   * web-app[verify auth.protected.extraction]
   * web-app[verify profile.route]
   * web-app[verify profile.auth_required]
   */
  test('authenticatedPage can access protected routes', async ({ authenticatedPage }) => {
    // Profile page requires authentication (at /me)
    await authenticatedPage.goto('/me');

    // Should load profile page instead of redirecting
    await expect(authenticatedPage).toHaveURL('/me');
    await expect(authenticatedPage.getByRole('heading', { name: 'My Profile' })).toBeVisible();
  });

  /**
   * web-app[verify auth.user.github_id]
   * web-app[verify auth.user.github_login]
   * web-app[verify auth.user.name]
   * web-app[verify auth.user.email]
   */
  test('mockUser has expected properties', async ({ mockUser }) => {
    // MockUser should have the expected shape
    expect(mockUser.id).toBeGreaterThan(0);
    expect(mockUser.login).toMatch(/^testuser_\d+$/);
    expect(mockUser.name).toMatch(/^Test User \d+$/);
    expect(mockUser.email).toMatch(/^test\d+@example\.com$/);
  });
});
