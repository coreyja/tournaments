-- Remove error tracking fields from jobs table

ALTER TABLE jobs
DROP COLUMN IF EXISTS error_count;

ALTER TABLE jobs
DROP COLUMN IF EXISTS last_error_message;

ALTER TABLE jobs
DROP COLUMN IF EXISTS last_failed_at;
