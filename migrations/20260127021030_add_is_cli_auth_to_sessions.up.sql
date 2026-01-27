-- Add is_cli_auth column to sessions table
-- This tracks whether the OAuth flow was initiated from the CLI
ALTER TABLE sessions ADD COLUMN is_cli_auth BOOLEAN NOT NULL DEFAULT FALSE;
