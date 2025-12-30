import { test, expect, createMockUser } from '../fixtures/test';

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

  test('different users can have same snake name', async ({ authenticatedPage, loginAsUser }) => {
    const sharedName = `Shared Name Snake ${Date.now()}`;

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

    // Log in as second user
    const secondUser = createMockUser('user2');
    await loginAsUser(authenticatedPage, secondUser);

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
