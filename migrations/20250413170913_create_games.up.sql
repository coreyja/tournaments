-- Add migration script here
CREATE TABLE
  games (
    game_id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    board_size TEXT NOT NULL, -- '7x7', '11x11', or '19x19'
    game_type TEXT NOT NULL, -- 'Standard', 'Royale', 'Constrictor', or 'Snail Mode'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
  );

-- Create a trigger to automatically update the updated_at column for games
CREATE TRIGGER update_games_updated_at BEFORE
UPDATE ON games FOR EACH ROW EXECUTE FUNCTION update_updated_at_column ();

-- Create join table between games and battlesnakes
CREATE TABLE
  game_battlesnakes (
    game_battlesnake_id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    game_id UUID NOT NULL REFERENCES games (game_id) ON DELETE CASCADE,
    battlesnake_id UUID NOT NULL REFERENCES battlesnakes (battlesnake_id) ON DELETE CASCADE,
    placement INTEGER, -- NULL if the game is not finished, otherwise the placement (1st, 2nd, etc.)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    -- Ensure each battlesnake can only be in a game once
    UNIQUE (game_id, battlesnake_id)
  );

-- Create indexes for faster lookups
CREATE INDEX game_battlesnakes_game_id_idx ON game_battlesnakes (game_id);

CREATE INDEX game_battlesnakes_battlesnake_id_idx ON game_battlesnakes (battlesnake_id);

-- Create a trigger to automatically update the updated_at column for game_battlesnakes
CREATE TRIGGER update_game_battlesnakes_updated_at BEFORE
UPDATE ON game_battlesnakes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column ();
