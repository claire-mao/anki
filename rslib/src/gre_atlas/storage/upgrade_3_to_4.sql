ALTER TABLE bl_question
ADD COLUMN source_name TEXT;
ALTER TABLE bl_question
ADD COLUMN source_section TEXT;
ALTER TABLE bl_question
ADD COLUMN generated_at_secs INTEGER;
ALTER TABLE bl_question
ADD COLUMN generation_confidence REAL;