-- Add timer and timeout columns to room_config (if not exist)
ALTER TABLE room_config ADD COLUMN turn_timer_seconds INTEGER DEFAULT 10;
ALTER TABLE room_config ADD COLUMN buzz_timer_seconds INTEGER DEFAULT 5;
ALTER TABLE room_config ADD COLUMN disconnect_timeout_secs INTEGER DEFAULT 300;
