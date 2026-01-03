DROP INDEX IF EXISTS idx_games_archived_at;
DROP INDEX IF EXISTS idx_games_engine_game_id;

ALTER TABLE games DROP COLUMN IF EXISTS gcs_path;
ALTER TABLE games DROP COLUMN IF EXISTS archived_at;
ALTER TABLE games DROP COLUMN IF EXISTS engine_game_id;
