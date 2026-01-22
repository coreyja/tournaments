-- Tracks batches of historical game backups for fork-join processing
CREATE TABLE backup_batches (
    id SERIAL PRIMARY KEY,
    -- Cursor for the NEXT batch (where to resume after this batch completes)
    next_cursor_created TIMESTAMP,
    next_cursor_id TEXT,
    -- Tracking completion
    jobs_enqueued INT NOT NULL,
    jobs_completed INT NOT NULL DEFAULT 0,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- Index for finding incomplete batches
CREATE INDEX idx_backup_batches_incomplete ON backup_batches (completed_at) WHERE completed_at IS NULL;
