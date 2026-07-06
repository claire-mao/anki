-- Schema v6: staged topic flashcard release batches after GRE practice.
CREATE TABLE IF NOT EXISTS bl_topic_flashcard_batch (
  topic TEXT NOT NULL,
  batch_index INTEGER NOT NULL,
  release_at_secs INTEGER NOT NULL,
  card_ids_json TEXT NOT NULL,
  released INTEGER NOT NULL DEFAULT 0,
  usn INTEGER NOT NULL DEFAULT 0,
  mtime_secs INTEGER NOT NULL,
  PRIMARY KEY (topic, batch_index)
);
CREATE INDEX IF NOT EXISTS ix_bl_topic_flashcard_batch_release
  ON bl_topic_flashcard_batch(released, release_at_secs);
