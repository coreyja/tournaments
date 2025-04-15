-- Add migration script here
-- Add flash message fields to sessions table
ALTER TABLE sessions
ADD COLUMN flash_message TEXT NULL,
ADD COLUMN flash_type TEXT NULL;

-- Create an index to quickly find sessions with flash messages
CREATE INDEX sessions_with_flash_idx ON sessions (flash_message)
WHERE
  flash_message IS NOT NULL;
