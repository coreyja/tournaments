-- Add latency tracking columns to snake_turns table
ALTER TABLE snake_turns ADD COLUMN latency_ms INTEGER;
ALTER TABLE snake_turns ADD COLUMN timed_out BOOLEAN NOT NULL DEFAULT FALSE;
