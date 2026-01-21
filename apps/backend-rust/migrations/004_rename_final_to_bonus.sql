-- Migration: 004_rename_final_to_bonus
-- Description: Rename final_* columns to bonus_* in room_config table
-- Created: 2026-01-20
--
-- This migration renames columns to match the new naming convention
-- where "Final" now refers to "Final Spin" phase and "Bonus" refers
-- to what was previously called the "Final" round.
--
-- SQLite 3.25+ supports ALTER TABLE RENAME COLUMN

-- Rename final_seconds to bonus_seconds (if it exists)
-- Note: This will fail gracefully if column doesn't exist or is already renamed
ALTER TABLE room_config RENAME COLUMN final_seconds TO bonus_seconds;

-- Rename final_jackpot to bonus_jackpot (if it exists)
ALTER TABLE room_config RENAME COLUMN final_jackpot TO bonus_jackpot;
