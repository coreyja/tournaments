-- Add migration script here
CREATE TABLE
  IF NOT EXISTS Jobs (
    job_id UUID PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    payload JSONB NOT NULL,
    priority INT NOT NULL,
    run_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    locked_at TIMESTAMPTZ,
    locked_by TEXT,
    context TEXT NOT NULL
  );
