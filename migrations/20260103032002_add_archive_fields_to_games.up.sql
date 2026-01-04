-- Link to Engine DB game ID (for games imported from legacy Engine)
ALTER TABLE games ADD COLUMN engine_game_id VARCHAR(255) UNIQUE;

-- When game was exported to GCS (NULL = not archived yet)
ALTER TABLE games ADD COLUMN archived_at TIMESTAMPTZ;

-- GCS path where the game is stored
ALTER TABLE games ADD COLUMN gcs_path TEXT;

CREATE INDEX idx_games_engine_game_id ON games(engine_game_id);
CREATE INDEX idx_games_archived_at ON games(archived_at);
