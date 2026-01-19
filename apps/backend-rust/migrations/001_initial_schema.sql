-- Migration: 001_initial_schema
-- Description: Initial database schema for Holiday Wheel backend
-- Created: 2026-01-19
--
-- This migration creates all core tables for the application:
-- - users: User accounts with authentication data
-- - rooms: Game room metadata
-- - packs: Puzzle pack definitions
-- - puzzles: Individual puzzles with categories
-- - used_puzzles: Tracking which puzzles have been used in each room
-- - room_config: Per-room game configuration
-- - passkey_credentials: WebAuthn/Passkey storage
-- - oauth_accounts: OAuth provider linkage (Google, Apple)
-- - webauthn_challenges: Temporary challenge storage for WebAuthn flows

-- ============================================
-- USERS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password_hash TEXT,                          -- NULL for OAuth-only users
    display_name TEXT NOT NULL,
    verified INTEGER NOT NULL DEFAULT 0,
    verification_token TEXT,
    verification_token_expires INTEGER,
    reset_token TEXT,
    reset_token_expires INTEGER,
    created_at INTEGER NOT NULL,
    last_login_at INTEGER,
    remember_token TEXT,
    is_admin INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_verification_token ON users(verification_token);
CREATE INDEX IF NOT EXISTS idx_users_remember_token ON users(remember_token);
CREATE INDEX IF NOT EXISTS idx_users_reset_token ON users(reset_token);

-- ============================================
-- ROOMS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS rooms (
    name TEXT PRIMARY KEY,
    created_by INTEGER,
    created_at INTEGER NOT NULL,
    last_activity_at INTEGER NOT NULL,
    is_public INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY(created_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_rooms_last_activity ON rooms(last_activity_at);
CREATE INDEX IF NOT EXISTS idx_rooms_created_by ON rooms(created_by);

-- ============================================
-- PACKS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS packs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Insert default pack
INSERT OR IGNORE INTO packs (id, name) VALUES (1, 'Default');

-- ============================================
-- PUZZLES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS puzzles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL,
    answer TEXT NOT NULL,
    pack_id INTEGER NOT NULL DEFAULT 1,
    enabled INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY(pack_id) REFERENCES packs(id)
);

CREATE INDEX IF NOT EXISTS idx_puzzles_pack_id ON puzzles(pack_id);
CREATE INDEX IF NOT EXISTS idx_puzzles_category ON puzzles(category);
CREATE INDEX IF NOT EXISTS idx_puzzles_enabled ON puzzles(enabled);

-- ============================================
-- USED PUZZLES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS used_puzzles (
    room_name TEXT NOT NULL,
    puzzle_id INTEGER NOT NULL,
    used_at INTEGER NOT NULL,
    PRIMARY KEY(room_name, puzzle_id),
    FOREIGN KEY(puzzle_id) REFERENCES puzzles(id)
);

CREATE INDEX IF NOT EXISTS idx_used_puzzles_room ON used_puzzles(room_name);

-- ============================================
-- ROOM CONFIG TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS room_config (
    room_name TEXT PRIMARY KEY,
    active_pack_id INTEGER,
    vowel_cost INTEGER,
    final_seconds INTEGER,
    final_jackpot INTEGER,
    prize_replace_csv TEXT,
    puzzle_display_seconds INTEGER,
    prize_wedge_names TEXT,
    disconnect_timeout_secs INTEGER DEFAULT 300,
    FOREIGN KEY(active_pack_id) REFERENCES packs(id)
);

-- ============================================
-- PASSKEY CREDENTIALS TABLE (WebAuthn)
-- ============================================
CREATE TABLE IF NOT EXISTS passkey_credentials (
    id TEXT PRIMARY KEY,                         -- credential ID (base64url)
    user_id INTEGER NOT NULL,
    public_key BLOB NOT NULL,                    -- COSE public key
    counter INTEGER NOT NULL DEFAULT 0,          -- signature counter
    transports TEXT,                             -- JSON array of transports
    created_at INTEGER NOT NULL,
    last_used_at INTEGER,
    device_name TEXT,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_passkey_user_id ON passkey_credentials(user_id);

-- ============================================
-- OAUTH ACCOUNTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS oauth_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    provider TEXT NOT NULL,                      -- 'google' or 'apple'
    provider_user_id TEXT NOT NULL,
    email TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(provider, provider_user_id)
);

CREATE INDEX IF NOT EXISTS idx_oauth_user_id ON oauth_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_oauth_provider ON oauth_accounts(provider, provider_user_id);

-- ============================================
-- WEBAUTHN CHALLENGES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS webauthn_challenges (
    challenge TEXT PRIMARY KEY,
    user_id INTEGER,
    email TEXT,
    type TEXT NOT NULL,                          -- 'registration' or 'authentication'
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_challenges_expires ON webauthn_challenges(expires_at);
