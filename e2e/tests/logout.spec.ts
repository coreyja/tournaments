import { test, expect } from '../fixtures/test';

/**
 * Logout Flow
 *
 * [verify auth.logout.route]
 * [verify auth.logout.redirect]
 * [verify auth.logout.session.disassociation]
 * [verify auth.protected.unauthorized]
 */
test.describe('Logout Flow', () => {
  /**
   * [verify auth.logout.route]
   * [verify auth.logout.redirect]
   */
  test('clicking logout redirects to homepage', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/');

    // Click the logout link
    await authenticatedPage.getByRole('link', { name: 'Logout' }).click();

    // Should redirect to homepage
    await expect(authenticatedPage).toHaveURL('/');
  });

  /**
   * [verify auth.logout.session.disassociation]
   * [verify homepage.unauth.message]
   * [verify homepage.unauth.login_link]
   */
  test('after logout, homepage shows login link', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/');

    // Verify we're logged in first
    await expect(authenticatedPage.getByRole('link', { name: 'Logout' })).toBeVisible();

    // Click logout
    await authenticatedPage.getByRole('link', { name: 'Logout' }).click();
    await authenticatedPage.waitForURL('/');

    // Should now see the login link instead
    await expect(authenticatedPage.getByRole('link', { name: 'Login with GitHub' })).toBeVisible();
    await expect(authenticatedPage.getByText('You are not logged in.')).toBeVisible();
  });

  /**
   * [verify auth.protected.unauthorized]
   * [verify auth.logout.session.disassociation]
   */
  test('protected routes return 401 after logout', async ({ authenticatedPage }) => {
    // First verify we can access /me while logged in
    await authenticatedPage.goto('/me');
    await expect(authenticatedPage.getByRole('heading', { name: 'My Profile' })).toBeVisible();

    // Now logout
    await authenticatedPage.goto('/');
    await authenticatedPage.getByRole('link', { name: 'Logout' }).click();
    await authenticatedPage.waitForURL('/');

    // Try to access /me again - should get 401
    const response = await authenticatedPage.goto('/me');
    expect(response?.status()).toBe(401);
  });
});
