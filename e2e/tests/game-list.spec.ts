import { test, expect } from '../fixtures/test';

test.describe('Game List', () => {
  test('shows games list page', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/games');

    await expect(authenticatedPage.getByRole('heading', { name: 'All Games' })).toBeVisible();
    // Either shows empty state or games table - both are valid
    const hasGames = await authenticatedPage.locator('table').isVisible();
    if (!hasGames) {
      await expect(authenticatedPage.getByText('No games have been created yet.')).toBeVisible();
    }
  });

  test('can navigate to create new game from list', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/games');
    await expect(authenticatedPage.getByRole('heading', { name: 'All Games' })).toBeVisible();

    await authenticatedPage.getByRole('link', { name: 'Create New Game' }).click();

    // Should redirect to a game flow page
    await expect(authenticatedPage).toHaveURL(/\/games\/flow\//);
  });

  test('displays created game in list with correct info', async ({ authenticatedPage }) => {
    // First create a battlesnake
    const snakeName = `List Test Snake ${Date.now()}`;
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(snakeName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/list-test');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Start game creation flow
    await authenticatedPage.goto('/games/new');
    await expect(authenticatedPage).toHaveURL(/\/games\/flow\//);

    // Add the battlesnake
    const snakeCard = authenticatedPage.locator('.card', { hasText: snakeName });
    await snakeCard.getByRole('button', { name: 'Add to Game' }).click();

    // Verify snake is selected
    await expect(authenticatedPage.getByText('You have selected 1 of 4 possible battlesnakes.')).toBeVisible();

    // Select board size and game type
    await authenticatedPage.getByLabel('Board Size').selectOption('19x19');
    await authenticatedPage.getByLabel('Game Type').selectOption('Royale');

    // Create the game
    await authenticatedPage.getByRole('button', { name: 'Create Game' }).click();

    // Should redirect to game details
    await expect(authenticatedPage).toHaveURL(/\/games\/[0-9a-f-]+$/);

    // Now go to list and verify the game appears
    await authenticatedPage.goto('/games');

    // Should see a game in the table (board size shows as just "19x19")
    // Use .first() since there may be multiple games from parallel tests
    await expect(authenticatedPage.getByRole('cell', { name: '19x19' }).first()).toBeVisible();
    await expect(authenticatedPage.getByRole('cell', { name: 'Royale' }).first()).toBeVisible();

    // Should have a View button
    await expect(authenticatedPage.getByRole('link', { name: 'View' }).first()).toBeVisible();
  });

  test('can view game details from list', async ({ authenticatedPage }) => {
    // First create a battlesnake
    const snakeName = `View Test Snake ${Date.now()}`;
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(snakeName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/view-test');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Create a game
    await authenticatedPage.goto('/games/new');
    const snakeCard = authenticatedPage.locator('.card', { hasText: snakeName });
    await snakeCard.getByRole('button', { name: 'Add to Game' }).click();
    await authenticatedPage.getByRole('button', { name: 'Create Game' }).click();

    // Go to list
    await authenticatedPage.goto('/games');

    // Click View on the first game
    await authenticatedPage.getByRole('link', { name: 'View' }).first().click();

    // Should be on game details page
    await expect(authenticatedPage).toHaveURL(/\/games\/[0-9a-f-]+$/);
    await expect(authenticatedPage.getByRole('heading', { name: 'Game Details' })).toBeVisible();
  });

  test('game details page shows battlesnakes and placements', async ({ authenticatedPage }) => {
    // Create a battlesnake
    const snakeName = `Details Test Snake ${Date.now()}`;
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(snakeName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/details-test');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Create a game
    await authenticatedPage.goto('/games/new');
    const snakeCard = authenticatedPage.locator('.card', { hasText: snakeName });
    await snakeCard.getByRole('button', { name: 'Add to Game' }).click();
    await authenticatedPage.getByRole('button', { name: 'Create Game' }).click();

    // Should be on game details page
    await expect(authenticatedPage).toHaveURL(/\/games\/[0-9a-f-]+$/);

    // Should show game results table
    await expect(authenticatedPage.getByRole('heading', { name: 'Game Results' })).toBeVisible();

    // Should show the snake name in the table
    await expect(authenticatedPage.getByText(snakeName)).toBeVisible();

    // Should show placement badge (1st place for single snake)
    await expect(authenticatedPage.getByText('1st Place')).toBeVisible();
  });

  test('game details shows board size and game type', async ({ authenticatedPage }) => {
    // Create a battlesnake
    const snakeName = `Config Test Snake ${Date.now()}`;
    await authenticatedPage.goto('/battlesnakes/new');
    await authenticatedPage.getByLabel('Name').fill(snakeName);
    await authenticatedPage.getByLabel('URL').fill('https://example.com/config-test');
    await authenticatedPage.getByLabel('Visibility').selectOption('public');
    await authenticatedPage.getByRole('button', { name: 'Create Battlesnake' }).click();

    // Create a game with specific settings
    await authenticatedPage.goto('/games/new');
    const snakeCard = authenticatedPage.locator('.card', { hasText: snakeName });
    await snakeCard.getByRole('button', { name: 'Add to Game' }).click();
    await authenticatedPage.getByLabel('Board Size').selectOption('7x7');
    await authenticatedPage.getByLabel('Game Type').selectOption('Constrictor');
    await authenticatedPage.getByRole('button', { name: 'Create Game' }).click();

    // Verify details page shows correct config
    await expect(authenticatedPage.getByText('Board Size: 7x7')).toBeVisible();
    await expect(authenticatedPage.getByText('Game Type: Constrictor')).toBeVisible();
  });

  test('game list requires authentication', async ({ page }) => {
    const response = await page.goto('/games');
    expect(response?.status()).toBe(401);
  });
});
