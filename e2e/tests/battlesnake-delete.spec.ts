import { test, expect } from '../fixtures/test';

/**
 * Battlesnake Delete
 *
 * [verify battlesnake.delete.route]
 * [verify battlesnake.delete.confirmation]
 * [verify battlesnake.delete.success_redirect]
 * [verify battlesnake.delete.cancel_preserves]
 */
test.describe('Battlesnake Delete', () => {
  /**
   * [verify battlesnake.delete.route]
   * [verify battlesnake.delete.confirmation]
   * [verify battlesnake.delete.success_redirect]
   */
  test('can delete a battlesnake from the list', async ({ authenticatedPage }) => {
    const uniqueName = `Delete Me Snake ${Date.now()}`;

    // Create a battlesnake first
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(uniqueName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/delete-me');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();
    await expect(authenticatedPage).toHaveURL('/battlesnakes');

    // Verify the snake exists
    await expect(authenticatedPage.getByText(uniqueName)).toBeVisible();

    // Set up dialog handler to accept the confirmation
    authenticatedPage.on('dialog', async (dialog) => {
      expect(dialog.type()).toBe('confirm');
      expect(dialog.message()).toContain('Are you sure');
      await dialog.accept();
    });

    // Click delete on the snake
    const snakeRow = authenticatedPage.locator('tr', { hasText: uniqueName });
    await snakeRow.getByRole('button', { name: 'Delete' }).click();

    // Should redirect to list and snake should be gone
    await expect(authenticatedPage).toHaveURL('/battlesnakes');
    await expect(authenticatedPage.getByText(uniqueName)).not.toBeVisible();
  });

  /**
   * [verify battlesnake.delete.cancel_preserves]
   */
  test('cancel delete keeps the battlesnake', async ({ authenticatedPage }) => {
    const uniqueName = `Keep Me Snake ${Date.now()}`;

    // Create a battlesnake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(uniqueName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/keep-me');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Set up dialog handler to dismiss (cancel) the confirmation
    authenticatedPage.on('dialog', async (dialog) => {
      await dialog.dismiss();
    });

    // Click delete but cancel
    const snakeRow = authenticatedPage.locator('tr', { hasText: uniqueName });
    await snakeRow.getByRole('button', { name: 'Delete' }).click();

    // Snake should still be visible (page didn't navigate)
    await expect(authenticatedPage.getByText(uniqueName)).toBeVisible();
  });

  /**
   * [verify battlesnake.delete.route]
   */
  test('deleting one snake does not affect others', async ({ authenticatedPage }) => {
    const keepName = `Keep This Snake ${Date.now()}`;
    const deleteName = `Delete This Snake ${Date.now()}`;

    // Create two battlesnakes
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(keepName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/keep');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(deleteName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/delete');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Both should be visible
    await expect(authenticatedPage.getByText(keepName)).toBeVisible();
    await expect(authenticatedPage.getByText(deleteName)).toBeVisible();

    // Accept delete dialog
    authenticatedPage.on('dialog', (dialog) => dialog.accept());

    // Delete one snake
    const deleteRow = authenticatedPage.locator('tr', { hasText: deleteName });
    await deleteRow.getByRole('button', { name: 'Delete' }).click();

    // Wait for redirect back to list
    await expect(authenticatedPage).toHaveURL('/battlesnakes');

    // Deleted snake should be gone, other should remain
    await expect(authenticatedPage.getByText(deleteName)).not.toBeVisible();
    await expect(authenticatedPage.getByText(keepName)).toBeVisible();
  });

  /**
   * [verify battlesnake.delete.success_redirect]
   * [verify battlesnake.list.empty_state]
   */
  test('delete returns to list with empty state when last snake deleted', async ({ authenticatedPage }) => {
    const uniqueName = `Last Snake ${Date.now()}`;

    // Create a battlesnake
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(uniqueName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/last');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Accept delete dialog
    authenticatedPage.on('dialog', (dialog) => dialog.accept());

    // Delete the snake
    const snakeRow = authenticatedPage.locator('tr', { hasText: uniqueName });
    await snakeRow.getByRole('button', { name: 'Delete' }).click();

    // Should show empty state
    await expect(authenticatedPage).toHaveURL('/battlesnakes');
    await expect(authenticatedPage.getByText("You don't have any battlesnakes yet.")).toBeVisible();
  });
});
