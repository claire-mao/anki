CREATE TABLE IF NOT EXISTS bl_session (
  id TEXT PRIMARY KEY,
  started_at_secs INTEGER NOT NULL,
  ended_at_secs INTEGER,
  source TEXT NOT NULL DEFAULT 'practice',
  usn INTEGER NOT NULL DEFAULT -1,
  mtime_secs INTEGER NOT NULL
);
ALTER TABLE bl_performance_attempt ADD COLUMN difficulty REAL;
CREATE INDEX IF NOT EXISTS ix_bl_attempt_session ON bl_performance_attempt(session_id);
CREATE INDEX IF NOT EXISTS ix_bl_session_started ON bl_session(started_at_secs);
