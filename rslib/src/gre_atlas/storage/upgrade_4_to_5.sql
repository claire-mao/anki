-- Schema v5: optional AI question-generation metadata + generation eval log.
-- Backward-compatible: new columns are nullable; existing seed/foundation and
-- template rows keep NULLs and continue to work unchanged.
ALTER TABLE bl_question
ADD COLUMN source_document TEXT;
ALTER TABLE bl_question
ADD COLUMN model_version TEXT;
ALTER TABLE bl_question
ADD COLUMN provenance TEXT;
ALTER TABLE bl_question
ADD COLUMN evaluation_status TEXT;
CREATE TABLE IF NOT EXISTS bl_generation_eval (
  id INTEGER PRIMARY KEY,
  candidate_id TEXT NOT NULL,
  topic TEXT NOT NULL,
  model_version TEXT NOT NULL,
  provenance TEXT NOT NULL,
  STATUS TEXT NOT NULL,
  reason TEXT NOT NULL DEFAULT '',
  confidence REAL,
  evaluated_at_secs INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS ix_bl_generation_eval_status ON bl_generation_eval(STATUS);
CREATE INDEX IF NOT EXISTS ix_bl_generation_eval_time ON bl_generation_eval(evaluated_at_secs);