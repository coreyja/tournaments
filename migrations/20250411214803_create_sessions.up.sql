-- Add migration script here
CREATE TABLE
  sessions (
    session_id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    user_id UUID REFERENCES users (user_id) NULL,
    github_oauth_state TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW () + INTERVAL '7 days')
  );

-- Create an index for finding sessions by user
CREATE INDEX sessions_user_id_idx ON sessions (user_id);

-- Create an index for cleaning up expired sessions
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at);

-- Add trigger to update the updated_at column automatically
CREATE TRIGGER update_sessions_updated_at BEFORE
UPDATE ON sessions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column ();
