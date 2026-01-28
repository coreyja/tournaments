-- Allow the same battlesnake to appear multiple times in a game
-- This enables scenarios like testing a snake against itself
ALTER TABLE game_battlesnakes DROP CONSTRAINT game_battlesnakes_game_id_battlesnake_id_key;
