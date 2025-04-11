-- Drop the users table
DROP TABLE IF EXISTS users;

-- Drop the function used for the updated_at trigger
DROP FUNCTION IF EXISTS update_updated_at_column;

-- We don't drop the uuid-ossp extension as it might be used by other tables
