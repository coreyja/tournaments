-- Add migration script here
CREATE TABLE
  IF NOT EXISTS Crons (
    cron_id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    last_run_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL,
      created_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL,
      updated_at TIMESTAMP
    WITH
      TIME ZONE NOT NULL
  );

CREATE UNIQUE INDEX idx_crons_name ON Crons (name);
