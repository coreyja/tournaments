-- Add error tracking fields to jobs table
-- Add migration script here

-- Add error_count column (tracks number of failures)
ALTER TABLE jobs
ADD COLUMN IF NOT EXISTS error_count INTEGER NOT NULL DEFAULT 0;

-- Add last_error_message column (stores most recent error message)
ALTER TABLE jobs
ADD COLUMN IF NOT EXISTS last_error_message TEXT;

-- Add last_failed_at column (stores timestamp of most recent failure)
ALTER TABLE jobs
ADD COLUMN IF NOT EXISTS last_failed_at TIMESTAMPTZ;
