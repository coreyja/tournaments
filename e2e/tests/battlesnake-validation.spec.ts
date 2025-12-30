import { test, expect } from '../fixtures/test';

test.describe('Battlesnake Validation', () => {
  test('cannot create battlesnake with duplicate name', async ({ authenticatedPage }) => {
    const duplicateName = `Duplicate Snake ${Date.now()}`;

    // Create first battlesnake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(duplicateName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/first');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();
    await expect(authenticatedPage).toHaveURL('/battlesnakes');
    await expect(authenticatedPage.getByText(duplicateName)).toBeVisible();

    // Try to create second with same name - should stay on form
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(duplicateName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/second');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Should stay on form page (not redirect to list)
    await expect(authenticatedPage).toHaveURL('/battlesnakes/new');
  });

  test('cannot update battlesnake to use duplicate name', async ({ authenticatedPage }) => {
    const firstName = `First Snake ${Date.now()}`;
    const secondName = `Second Snake ${Date.now()}`;

    // Create first battlesnake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(firstName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/first');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Create second battlesnake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(secondName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/second');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Try to rename second to first's name
    const secondRow = authenticatedPage.locator('tr', { hasText: secondName });
    await secondRow.getByRole('link', { name: 'Edit', exact: true }).click();
    await authenticatedPage.getByLabel('Name').fill(firstName);
    await authenticatedPage.getByRole('button', { name: 'Update Battlesnake' }).click();

    // Should stay on edit form (not redirect to list)
    await expect(authenticatedPage).toHaveURL(/\/battlesnakes\/.*\/edit/);
  });

  test('can use same name after deleting original', async ({ authenticatedPage }) => {
    const reuseName = `Reuse Name Snake ${Date.now()}`;

    // Create battlesnake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(reuseName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/original');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();
    await expect(authenticatedPage.getByText(reuseName)).toBeVisible();

    // Delete it
    authenticatedPage.on('dialog', (dialog) => dialog.accept());
    const snakeRow = authenticatedPage.locator('tr', { hasText: reuseName });
    await snakeRow.getByRole('button', { name: 'Delete' }).click();
    await expect(authenticatedPage.getByText(reuseName)).not.toBeVisible();

    // Create new one with same name - should succeed
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(reuseName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/new');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Should redirect to list with new snake
    await expect(authenticatedPage).toHaveURL('/battlesnakes');
    await expect(authenticatedPage.getByText(reuseName)).toBeVisible();
  });

  test('different users can have same snake name', async ({ authenticatedPage, browser }) => {
    const sharedName = `Shared Name Snake ${Date.now()}`;
    const MOCK_GITHUB_URL = `http://localhost:${process.env.MOCK_GITHUB_PORT || '8081'}`;

    // First user creates a snake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(sharedName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/user1');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();
    await expect(authenticatedPage).toHaveURL('/battlesnakes');
    await expect(authenticatedPage.getByText(sharedName)).toBeVisible();

    // Log out first user
    await authenticatedPage.goto('/auth/logout');

    // Create second user with different ID
    const secondUserId = Date.now() * 1000 + Math.floor(Math.random() * 1000) + 1;
    const secondUser = {
      id: secondUserId,
      login: `testuser2_${secondUserId}`,
      name: `Test User 2`,
      email: `test2_${secondUserId}@example.com`,
      avatar_url: 'https://example.com/avatar2.png',
    };

    // Set up route handler for second user's OAuth flow
    await authenticatedPage.route('**/auth/github', async (route) => {
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
            body: JSON.stringify({ state, user: secondUser }),
          });
        }
      }

      await route.fulfill({
        status: nativeResponse.status,
        headers: Object.fromEntries(nativeResponse.headers.entries()),
        body: await nativeResponse.text(),
      });
    });

    // Log in as second user
    await authenticatedPage.goto('/auth/github');
    await authenticatedPage.waitForURL('/', { timeout: 10000 });
    await expect(authenticatedPage.getByText(`Welcome, ${secondUser.login}!`)).toBeVisible({ timeout: 5000 });

    // Second user creates snake with same name - should succeed
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(sharedName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/user2');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Should succeed and redirect to list
    await expect(authenticatedPage).toHaveURL('/battlesnakes');
    await expect(authenticatedPage.getByText(sharedName)).toBeVisible();
  });
});
