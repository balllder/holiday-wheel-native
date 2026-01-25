-- OAuth state persistence for CSRF protection
-- Stores temporary state during OAuth redirects (10 minute expiration)

CREATE TABLE IF NOT EXISTS oauth_states (
    state TEXT PRIMARY KEY,
    user_data TEXT NOT NULL,   -- JSON: redirect_uri, etc.
    provider TEXT NOT NULL,     -- 'apple', 'google', etc.
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

-- Index for cleanup of expired states
CREATE INDEX IF NOT EXISTS idx_oauth_states_expires_at ON oauth_states(expires_at);
