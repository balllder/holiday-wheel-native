-- Migration: 002_additional_indexes
-- Description: Additional performance indexes
-- Created: 2026-01-19
--
-- This migration adds indexes that may have been missed or are beneficial
-- for common query patterns.

-- Index for looking up users by verification token expiry (cleanup queries)
CREATE INDEX IF NOT EXISTS idx_users_verification_expires ON users(verification_token_expires)
    WHERE verification_token IS NOT NULL;

-- Index for looking up users by reset token expiry (cleanup queries)
CREATE INDEX IF NOT EXISTS idx_users_reset_expires ON users(reset_token_expires)
    WHERE reset_token IS NOT NULL;

-- Index for admin user lookups
CREATE INDEX IF NOT EXISTS idx_users_is_admin ON users(is_admin)
    WHERE is_admin = 1;

-- Index for active/enabled puzzles with pack (common query pattern)
CREATE INDEX IF NOT EXISTS idx_puzzles_pack_enabled ON puzzles(pack_id, enabled)
    WHERE enabled = 1;

-- Index for room config by pack (for pack deletion checks)
CREATE INDEX IF NOT EXISTS idx_room_config_pack ON room_config(active_pack_id);

-- Index for cleaning up expired challenges
CREATE INDEX IF NOT EXISTS idx_challenges_type_expires ON webauthn_challenges(type, expires_at);
