-- Add up migration script here
-- Add status field to games table
ALTER TABLE games
ADD COLUMN status TEXT NOT NULL DEFAULT 'waiting';

-- Create an index for the status column to improve query performance
CREATE INDEX games_status_idx ON games (status);
