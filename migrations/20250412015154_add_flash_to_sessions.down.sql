-- Drop the index
DROP INDEX IF EXISTS sessions_with_flash_idx;

-- Remove flash message columns from sessions table
ALTER TABLE sessions
DROP COLUMN IF EXISTS flash_message,
DROP COLUMN IF EXISTS flash_type;
