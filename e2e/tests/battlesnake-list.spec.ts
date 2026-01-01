import { test, expect } from '../fixtures/test';

test.describe('Battlesnake List', () => {
  test('displays empty state when user has no battlesnakes', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/battlesnakes');

    // web-app[verify battlesnake.list.empty-state]
    await expect(authenticatedPage.getByRole('heading', { name: 'Your Battlesnakes' })).toBeVisible();
    await expect(authenticatedPage.getByText("You don't have any battlesnakes yet.")).toBeVisible();
  });

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
    // web-app[verify battlesnake.list.display-name]
    await expect(authenticatedPage.getByText(uniqueName)).toBeVisible();
    await expect(authenticatedPage.getByText('https://example.com/snake')).toBeVisible();
    // web-app[verify battlesnake.visibility.public]
    await expect(authenticatedPage.getByText('Public')).toBeVisible();
  });

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
    // web-app[verify battlesnake.list.display-visibility]
    await expect(publicRow.locator('.badge', { hasText: 'Public' })).toBeVisible();
    await expect(privateRow.locator('.badge', { hasText: 'Private' })).toBeVisible();
  });

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
    // web-app[verify battlesnake.list.edit-button]
    await expect(snakeRow.getByRole('link', { name: 'Edit' })).toBeVisible();
    // web-app[verify battlesnake.list.delete-button]
    await expect(snakeRow.getByRole('button', { name: 'Delete' })).toBeVisible();
  });

  test('has Add New Battlesnake button', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/battlesnakes');

    const addButton = authenticatedPage.getByRole('link', { name: 'Add New Battlesnake' });
    // web-app[verify battlesnake.list.add-button]
    await expect(addButton).toBeVisible();
    await expect(addButton).toHaveAttribute('href', '/battlesnakes/new');
  });

  test('list page requires authentication', async ({ page }) => {
    const response = await page.goto('/battlesnakes');
    // web-app[verify battlesnake.list.auth-required]
    expect(response?.status()).toBe(401);
  });
});
