-- Add migration script here
-- Create a dedicated table for game creation flows
CREATE TABLE
  game_flows (
    flow_id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    board_size TEXT NOT NULL DEFAULT '11x11', -- Default to medium board
    game_type TEXT NOT NULL DEFAULT 'Standard', -- Default to standard game type
    selected_battlesnakes UUID[] NOT NULL DEFAULT '{}', -- Array of battlesnake IDs
    search_query TEXT DEFAULT NULL,
    user_id UUID NOT NULL REFERENCES users (user_id) ON DELETE CASCADE, -- Owner of the flow
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
  );

-- Create indexes
CREATE INDEX game_flows_user_id_idx ON game_flows (user_id);

-- Create a trigger to automatically update the updated_at column
CREATE TRIGGER update_game_flows_updated_at BEFORE
UPDATE ON game_flows FOR EACH ROW EXECUTE FUNCTION update_updated_at_column ();
