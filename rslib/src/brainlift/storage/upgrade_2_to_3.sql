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
