import { test, expect } from '../fixtures/test';

/**
 * Battlesnake List
 *
 * web-app[verify battlesnake.list.route]
 * web-app[verify battlesnake.list.auth_required]
 * web-app[verify battlesnake.list.empty_state]
 * web-app[verify battlesnake.list.display_name]
 * web-app[verify battlesnake.list.display_url]
 * web-app[verify battlesnake.list.display_visibility]
 * web-app[verify battlesnake.list.edit_button]
 * web-app[verify battlesnake.list.delete_button]
 * web-app[verify battlesnake.list.add_button]
 */
test.describe('Battlesnake List', () => {
  /**
   * web-app[verify battlesnake.list.empty_state]
   */
  test('displays empty state when user has no battlesnakes', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/battlesnakes');

    await expect(authenticatedPage.getByRole('heading', { name: 'Your Battlesnakes' })).toBeVisible();
    await expect(authenticatedPage.getByText("You don't have any battlesnakes yet.")).toBeVisible();
  });

  /**
   * web-app[verify battlesnake.list.display_name]
   * web-app[verify battlesnake.list.display_url]
   * web-app[verify battlesnake.list.display_visibility]
   */
  test('displays battlesnakes after creation', async ({ authenticatedPage }) => {
    const uniqueName = `List Test Snake ${Date.now()}`;

    // First create a battlesnake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(uniqueName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/snake');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Should be on the list page
    await expect(authenticatedPage).toHaveURL('/battlesnakes');

    // Battlesnake should appear in the table
    await expect(authenticatedPage.getByText(uniqueName)).toBeVisible();
    await expect(authenticatedPage.getByText('https://example.com/snake')).toBeVisible();
    await expect(authenticatedPage.getByText('Public')).toBeVisible();
  });

  /**
   * web-app[verify battlesnake.list.display_visibility]
   */
  test('shows correct visibility badges', async ({ authenticatedPage }) => {
    const publicName = `Public Snake ${Date.now()}`;
    const privateName = `Private Snake ${Date.now()}`;

    // Create a public snake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(publicName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/public');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();
    await expect(authenticatedPage).toHaveURL('/battlesnakes');

    // Create a private snake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(privateName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/private');
    await authenticatedPage.getByLabel('Visibility').selectOption('private');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();
    await expect(authenticatedPage).toHaveURL('/battlesnakes');

    // Both should be visible with correct badges
    const publicRow = authenticatedPage.locator('tr', { hasText: publicName });
    const privateRow = authenticatedPage.locator('tr', { hasText: privateName });

    // Check for badge elements specifically (they have the badge class)
    await expect(publicRow.locator('.badge', { hasText: 'Public' })).toBeVisible();
    await expect(privateRow.locator('.badge', { hasText: 'Private' })).toBeVisible();
  });

  /**
   * web-app[verify battlesnake.list.edit_button]
   * web-app[verify battlesnake.list.delete_button]
   */
  test('shows edit and delete buttons for each snake', async ({ authenticatedPage }) => {
    const uniqueName = `Actions Test Snake ${Date.now()}`;

    // Create a snake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(uniqueName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/actions');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Find the row with our snake
    const snakeRow = authenticatedPage.locator('tr', { hasText: uniqueName });

    // Should have Edit and Delete buttons
    await expect(snakeRow.getByRole('link', { name: 'Edit' })).toBeVisible();
    await expect(snakeRow.getByRole('button', { name: 'Delete' })).toBeVisible();
  });

  /**
   * web-app[verify battlesnake.list.add_button]
   */
  test('has Add New Battlesnake button', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/battlesnakes');

    const addButton = authenticatedPage.getByRole('link', { name: 'Add New Battlesnake' });
    await expect(addButton).toBeVisible();
    await expect(addButton).toHaveAttribute('href', '/battlesnakes/new');
  });

  /**
   * web-app[verify battlesnake.list.auth_required]
   */
  test('list page requires authentication', async ({ page }) => {
    const response = await page.goto('/battlesnakes');
    expect(response?.status()).toBe(401);
  });
});
