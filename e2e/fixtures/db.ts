import { Pool } from 'pg';

// Use DATABASE_URL from environment or fall back to local test database
const TEST_DATABASE_URL = process.env.DATABASE_URL || 'postgresql://localhost:5432/tournaments_test';

let pool: Pool | null = null;

function getPool(): Pool {
  if (!pool) {
    pool = new Pool({ connectionString: TEST_DATABASE_URL });
  }
  return pool;
}

export async function closePool(): Promise<void> {
  if (pool) {
    await pool.end();
    pool = null;
  }
}

export async function query<T = unknown>(text: string, params?: unknown[]): Promise<T[]> {
  const result = await getPool().query(text, params);
  return result.rows as T[];
}
