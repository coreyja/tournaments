import { query, queryOne } from './db';

export const SESSION_COOKIE_NAME = 'tournaments_session_id';

export interface TestUser {
  user_id: string;
  external_github_id: number;
  github_login: string;
  github_avatar_url: string | null;
  github_name: string | null;
  github_email: string | null;
}

export interface TestSession {
  session_id: string;
  user_id: string | null;
}

/**
 * Create a test user directly in the database.
 * Each call creates a unique user with a random github ID.
 */
export async function createTestUser(overrides?: Partial<{
  github_login: string;
  github_name: string;
  github_email: string;
  github_avatar_url: string;
}>): Promise<TestUser> {
  // Use random number + timestamp for uniqueness across parallel workers
  const uniqueId = Date.now() * 1000 + Math.floor(Math.random() * 1000);

  const user = await queryOne<TestUser>(
    `INSERT INTO users (
      external_github_id,
      github_login,
      github_avatar_url,
      github_name,
      github_email,
      github_access_token
    ) VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING
      user_id,
      external_github_id,
      github_login,
      github_avatar_url,
      github_name,
      github_email`,
    [
      uniqueId,
      overrides?.github_login ?? `testuser_${uniqueId}`,
      overrides?.github_avatar_url ?? null,
      overrides?.github_name ?? `Test User ${uniqueId}`,
      overrides?.github_email ?? `test${uniqueId}@example.com`,
      'fake_access_token', // Required by schema but not used in tests
    ]
  );

  if (!user) {
    throw new Error('Failed to create test user');
  }

  return user;
}

/**
 * Create a session for a test user.
 */
export async function createTestSession(userId: string): Promise<TestSession> {
  const session = await queryOne<TestSession>(
    `INSERT INTO sessions (user_id)
    VALUES ($1)
    RETURNING session_id, user_id`,
    [userId]
  );

  if (!session) {
    throw new Error('Failed to create test session');
  }

  return session;
}

/**
 * Create a test user with an authenticated session.
 * Returns both the user and session info.
 */
export async function createAuthenticatedUser(overrides?: Parameters<typeof createTestUser>[0]): Promise<{
  user: TestUser;
  session: TestSession;
}> {
  const user = await createTestUser(overrides);
  const session = await createTestSession(user.user_id);
  return { user, session };
}

/**
 * Delete a test user and all associated data.
 */
export async function deleteTestUser(userId: string): Promise<void> {
  // Delete sessions first (foreign key constraint)
  await query('DELETE FROM sessions WHERE user_id = $1', [userId]);
  // Delete battlesnakes owned by user
  await query('DELETE FROM battlesnakes WHERE user_id = $1', [userId]);
  // Delete the user
  await query('DELETE FROM users WHERE user_id = $1', [userId]);
}

/**
 * Delete a test session.
 */
export async function deleteTestSession(sessionId: string): Promise<void> {
  await query('DELETE FROM sessions WHERE session_id = $1', [sessionId]);
}

/**
 * Clean up all test data created during tests.
 * Call this in afterAll or globalTeardown.
 */
export async function cleanupTestData(): Promise<void> {
  // Delete all sessions for test users (users with login starting with 'testuser_')
  await query(`
    DELETE FROM sessions
    WHERE user_id IN (
      SELECT user_id FROM users WHERE github_login LIKE 'testuser_%'
    )
  `);

  // Delete all battlesnakes for test users
  await query(`
    DELETE FROM battlesnakes
    WHERE user_id IN (
      SELECT user_id FROM users WHERE github_login LIKE 'testuser_%'
    )
  `);

  // Delete all test users
  await query(`DELETE FROM users WHERE github_login LIKE 'testuser_%'`);
}
