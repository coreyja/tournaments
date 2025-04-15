-- Add migration script here
-- Add visibility column to battlesnakes table with default value of 'public'
ALTER TABLE battlesnakes
ADD COLUMN visibility TEXT NOT NULL DEFAULT 'public';

-- Create an index on visibility for faster filtering
CREATE INDEX battlesnakes_visibility_idx ON battlesnakes (visibility);
