import { test, expect } from '../fixtures/test';

test.describe('Authentication Fixtures', () => {
  test('authenticatedPage shows logged in user on homepage', async ({ authenticatedPage, testUser }) => {
    // authenticatedPage starts at home page after login redirect
    await authenticatedPage.goto('/');

    // Should show the welcome message with user's github login
    await expect(authenticatedPage.getByText(`Welcome, ${testUser.github_login}!`)).toBeVisible();
  });

  test('authenticatedPage can access protected routes', async ({ authenticatedPage }) => {
    // Profile page requires authentication (at /me)
    await authenticatedPage.goto('/me');

    // Should load profile page instead of redirecting
    await expect(authenticatedPage).toHaveURL('/me');
    await expect(authenticatedPage.getByRole('heading', { name: 'My Profile' })).toBeVisible();
  });

  test('testUser and testSession have valid UUIDs', async ({ testUser, testSession }) => {
    // UUID format validation
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

    expect(testUser.user_id).toMatch(uuidRegex);
    expect(testSession.session_id).toMatch(uuidRegex);
    expect(testSession.user_id).toBe(testUser.user_id);
  });
});
