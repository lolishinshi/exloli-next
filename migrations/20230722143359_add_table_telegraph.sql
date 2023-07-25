-- Add up migration script here
CREATE TABLE telegraph (
    gallery_id INTEGER PRIMARY KEY NOT NULL,
    url TEXT NOT NULL
);

INSERT INTO telegraph (gallery_id, url) SELECT gallery_id, telegraph FROM message GROUP BY gallery_id;

ALTER TABLE message DROP COLUMN telegraph;
ALTER TABLE message ADD COLUMN channel_id TEXT NOT NULL DEFAULT "exlolicon";