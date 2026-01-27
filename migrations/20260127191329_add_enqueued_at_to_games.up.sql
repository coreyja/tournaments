-- Add enqueued_at timestamp to track when a game was enqueued for processing
ALTER TABLE games ADD COLUMN enqueued_at TIMESTAMPTZ;
