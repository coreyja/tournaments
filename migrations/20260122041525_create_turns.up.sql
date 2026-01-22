-- Create turns table to store game state per turn
CREATE TABLE
  turns (
    turn_id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    game_id UUID NOT NULL REFERENCES games (game_id) ON DELETE CASCADE,
    turn_number INTEGER NOT NULL,
    frame_data JSONB, -- NULL until all snakes have moved, then filled with PascalCase frame data
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    -- Each turn number must be unique per game
    UNIQUE (game_id, turn_number)
  );

-- Create indexes for efficient queries
CREATE INDEX turns_game_id_idx ON turns (game_id);
CREATE INDEX turns_game_id_turn_number_idx ON turns (game_id, turn_number);

-- Create snake_turns table to store individual snake moves
CREATE TABLE
  snake_turns (
    snake_turn_id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    turn_id UUID NOT NULL REFERENCES turns (turn_id) ON DELETE CASCADE,
    game_battlesnake_id UUID NOT NULL REFERENCES game_battlesnakes (game_battlesnake_id) ON DELETE CASCADE,
    direction TEXT NOT NULL, -- 'up', 'down', 'left', 'right'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    -- Each snake can only have one move per turn
    UNIQUE (turn_id, game_battlesnake_id)
  );

-- Create indexes for efficient queries
CREATE INDEX snake_turns_turn_id_idx ON snake_turns (turn_id);
CREATE INDEX snake_turns_game_battlesnake_id_idx ON snake_turns (game_battlesnake_id);
