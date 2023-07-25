-- Add up migration script here
UPDATE gallery SET deleted = 1 WHERE id IN (SELECT gallery_id FROM poll WHERE score == -1.0);
ALTER TABLE gallery ADD favorite INTEGER;
