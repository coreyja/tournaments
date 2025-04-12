-- Add migration script here
-- Create battlesnakes table that belongs to a user
CREATE TABLE
  battlesnakes (
    battlesnake_id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    user_id UUID NOT NULL REFERENCES users (user_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
  );

-- Create an index on user_id for faster lookups
CREATE INDEX battlesnakes_user_id_idx ON battlesnakes (user_id);

-- Create a trigger to automatically update the updated_at column
CREATE TRIGGER update_battlesnakes_updated_at BEFORE
UPDATE ON battlesnakes FOR EACH ROW EXECUTE FUNCTION update_updated_at_column ();
