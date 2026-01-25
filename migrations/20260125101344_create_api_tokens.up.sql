-- Create api_tokens table for CLI/API authentication

CREATE TABLE api_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- Index for looking up tokens by hash (primary auth lookup)
CREATE INDEX idx_api_tokens_hash ON api_tokens(token_hash);

-- Index for listing a user's tokens
CREATE INDEX idx_api_tokens_user_id ON api_tokens(user_id);
