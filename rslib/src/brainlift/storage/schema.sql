CREATE TABLE IF NOT EXISTS bl_meta (KEY TEXT PRIMARY KEY, val TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS bl_question (
  id TEXT PRIMARY KEY,
  topic TEXT NOT NULL,
  section TEXT NOT NULL,
  format TEXT NOT NULL,
  stem TEXT NOT NULL,
  choices_json TEXT,
  correct_answer TEXT NOT NULL,
  explanation TEXT NOT NULL,
  difficulty REAL,
  usn INTEGER NOT NULL DEFAULT 0,
  mtime_secs INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS bl_performance_attempt (
  id INTEGER PRIMARY KEY,
  question_id TEXT NOT NULL,
  topic TEXT NOT NULL,
  answered_at_secs INTEGER NOT NULL,
  answer TEXT NOT NULL,
  correct INTEGER NOT NULL,
  response_time_ms INTEGER NOT NULL,
  confidence INTEGER,
  session_id TEXT,
  usn INTEGER NOT NULL DEFAULT -1,
  mtime_secs INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS ix_bl_attempt_topic ON bl_performance_attempt(topic);
CREATE INDEX IF NOT EXISTS ix_bl_attempt_time ON bl_performance_attempt(answered_at_secs);
CREATE INDEX IF NOT EXISTS ix_bl_attempt_usn ON bl_performance_attempt(usn);