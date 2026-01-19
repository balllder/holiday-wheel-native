-- Migration: 003_add_avatar
-- Description: Add avatar support for user profiles
-- Created: 2026-01-19
--
-- This migration adds an avatar_id column to the users table
-- to store the selected avatar for each player profile.
-- Avatar IDs correspond to predefined avatar options in the UI.

-- Add avatar_id column to users table (default: 1 = default avatar)
ALTER TABLE users ADD COLUMN avatar_id INTEGER NOT NULL DEFAULT 1;

-- Create index for potential avatar-based queries
CREATE INDEX IF NOT EXISTS idx_users_avatar ON users(avatar_id);
