-- Remove is_cli_auth column from sessions table
ALTER TABLE sessions DROP COLUMN is_cli_auth;
