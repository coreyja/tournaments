-- Add down migration script here
-- Drop the index first
DROP INDEX IF EXISTS games_status_idx;

-- Remove the status column
ALTER TABLE games
DROP COLUMN status;
