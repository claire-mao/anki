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
CREATE TABLE IF NOT EXISTS bl_session (
  id TEXT PRIMARY KEY,
  started_at_secs INTEGER NOT NULL,
  ended_at_secs INTEGER,
  source TEXT NOT NULL DEFAULT 'practice',
  usn INTEGER NOT NULL DEFAULT -1,
  mtime_secs INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS bl_performance_attempt (
  id INTEGER PRIMARY KEY,
  question_id TEXT NOT NULL,
  topic TEXT NOT NULL,
  difficulty REAL,
  answered_at_secs INTEGER NOT NULL,
  answer TEXT NOT NULL,
  correct INTEGER NOT NULL,
  response_time_ms INTEGER NOT NULL,
  confidence INTEGER,
  session_id TEXT,
  usn INTEGER NOT NULL DEFAULT -1,
  mtime_secs INTEGER NOT NULL,
  FOREIGN KEY (question_id) REFERENCES bl_question(id),
  FOREIGN KEY (session_id) REFERENCES bl_session(id)
);
CREATE INDEX IF NOT EXISTS ix_bl_question_topic ON bl_question(topic);
CREATE INDEX IF NOT EXISTS ix_bl_attempt_topic ON bl_performance_attempt(topic);
CREATE INDEX IF NOT EXISTS ix_bl_attempt_time ON bl_performance_attempt(answered_at_secs);
CREATE INDEX IF NOT EXISTS ix_bl_attempt_usn ON bl_performance_attempt(usn);
CREATE INDEX IF NOT EXISTS ix_bl_attempt_session ON bl_performance_attempt(session_id);
CREATE INDEX IF NOT EXISTS ix_bl_session_started ON bl_session(started_at_secs);
CREATE TABLE IF NOT EXISTS bl_readiness_prediction (
  id INTEGER PRIMARY KEY,
  predicted_at_secs INTEGER NOT NULL,
  projected_score REAL NOT NULL,
  projected_score_low REAL,
  projected_score_high REAL,
  memory_score REAL NOT NULL,
  performance_score REAL NOT NULL,
  coverage_ratio REAL NOT NULL,
  confidence_level TEXT NOT NULL,
  model_version TEXT NOT NULL DEFAULT 'readiness_v1',
  outcome_score REAL,
  outcome_observed_at_secs INTEGER,
  outcome_memory_score REAL,
  outcome_performance_score REAL,
  practice_correct INTEGER,
  practice_total INTEGER,
  usn INTEGER NOT NULL DEFAULT -1,
  mtime_secs INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS ix_bl_readiness_pred_time ON bl_readiness_prediction(predicted_at_secs);
CREATE INDEX IF NOT EXISTS ix_bl_readiness_pred_resolved ON bl_readiness_prediction(outcome_score);
