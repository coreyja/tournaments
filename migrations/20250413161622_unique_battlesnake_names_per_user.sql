-- Add migration script here
-- Add unique constraint on user_id and name
CREATE UNIQUE INDEX unique_battlesnake_name_per_user ON battlesnakes (user_id, name);
