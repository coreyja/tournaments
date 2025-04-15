-- Drop migration script here
-- Drop the index first
DROP INDEX IF EXISTS battlesnakes_visibility_idx;

-- Then drop the column
ALTER TABLE battlesnakes
DROP COLUMN visibility;
