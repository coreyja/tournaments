import { test, expect } from '../fixtures/test';

/**
 * Create Battlesnake
 *
 * [verify battlesnake.create.form_route]
 * [verify battlesnake.create.form_auth_required]
 * [verify battlesnake.create.post_route]
 * [verify battlesnake.create.fields]
 * [verify battlesnake.create.success_redirect]
 * [verify battlesnake.create.success_flash]
 */
test.describe('Create Battlesnake', () => {
  /**
   * [verify battlesnake.create.form_route]
   * [verify battlesnake.create.fields]
   * [verify battlesnake.create.success_redirect]
   */
  test('can create a battlesnake with valid data', async ({ authenticatedPage }) => {
    const uniqueName = `Test Snake ${Date.now()}`;
    const snakeUrl = 'https://example.com/my-snake';

    // Navigate to create form
    await authenticatedPage.goto('/battlesnakes/new');
    await expect(authenticatedPage.getByRole('heading', { name: 'Add New Battlesnake' })).toBeVisible();

    // Fill in the form
    await authenticatedPage.getByLabel('Name').fill(uniqueName);
    await authenticatedPage.getByLabel('URL').fill(snakeUrl);
    await authenticatedPage.getByLabel('Visibility').selectOption('public');

    // Submit the form
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Should redirect to /battlesnakes
    await expect(authenticatedPage).toHaveURL('/battlesnakes');

    // New battlesnake should appear in the list
    await expect(authenticatedPage.getByText(uniqueName)).toBeVisible();
  });

  /**
   * [verify battlesnake.model.visibility]
   */
  test('can create a private battlesnake', async ({ authenticatedPage }) => {
    const uniqueName = `Private Snake ${Date.now()}`;
    const snakeUrl = 'https://example.com/private-snake';

    await authenticatedPage.goto('/battlesnakes/new');

    await authenticatedPage.getByLabel('Name').fill(uniqueName);
    await authenticatedPage.getByLabel('URL').fill(snakeUrl);
    await authenticatedPage.getByLabel('Visibility').selectOption('private');

    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    await expect(authenticatedPage).toHaveURL('/battlesnakes');
    await expect(authenticatedPage.getByText(uniqueName)).toBeVisible();
  });

  /**
   * [verify battlesnake.create.success_flash]
   */
  test('shows success flash message after creating battlesnake', async ({ authenticatedPage }) => {
    const uniqueName = `Flash Test Snake ${Date.now()}`;

    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(uniqueName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/flash-test');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    await expect(authenticatedPage).toHaveURL('/battlesnakes');

    // Should see success flash message (check for the alert container with success styling)
    const successAlert = authenticatedPage.locator('.alert-success');
    await expect(successAlert).toBeVisible();
    await expect(successAlert).toContainText('Battlesnake created successfully!');
  });

  /**
   * [verify battlesnake.create.form_auth_required]
   */
  test('new form requires authentication', async ({ page }) => {
    // Without authentication, should get 401
    const response = await page.goto('/battlesnakes/new');
    expect(response?.status()).toBe(401);
  });
});
