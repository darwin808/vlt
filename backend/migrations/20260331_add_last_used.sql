ALTER TABLE secrets ADD COLUMN last_used_at TEXT;
CREATE INDEX IF NOT EXISTS idx_secrets_last_used ON secrets(last_used_at);
