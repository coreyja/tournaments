-- Add migration script here
-- Drop the join table first (since it references the games table)
DROP TABLE IF EXISTS game_battlesnakes;

-- Drop the games table
DROP TABLE IF EXISTS games;
