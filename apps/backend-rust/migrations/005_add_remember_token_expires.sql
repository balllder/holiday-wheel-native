-- Migration: Add remember_token_expires column for session expiration
-- This allows tokens to expire after a set period (e.g., 30 days)

ALTER TABLE users ADD COLUMN remember_token_expires INTEGER;

-- Index for efficient expired token cleanup
CREATE INDEX IF NOT EXISTS idx_users_remember_token_expires ON users(remember_token_expires);
