-- Remove latency tracking columns from snake_turns table
ALTER TABLE snake_turns DROP COLUMN latency_ms;
ALTER TABLE snake_turns DROP COLUMN timed_out;
