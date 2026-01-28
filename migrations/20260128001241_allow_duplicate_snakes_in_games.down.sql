-- Re-add unique constraint (will fail if duplicates exist)
ALTER TABLE game_battlesnakes ADD CONSTRAINT game_battlesnakes_game_id_battlesnake_id_key UNIQUE (game_id, battlesnake_id);
