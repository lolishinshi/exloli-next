-- Add migration script here
ALTER TABLE message RENAME TO _del_message;
DROP INDEX message_gallery_id_idx;
DROP INDEX message_publish_date_idx;

CREATE TABLE message (
     id INTEGER NOT NULL,
     channel_id TEXT NOT NULL,
     gallery_id INTEGER NOT NULL,
     publish_date DATE NOT NULL,
     PRIMARY KEY (id, channel_id)
);
CREATE INDEX message_gallery_id_idx ON message (gallery_id);
CREATE INDEX message_publish_date_idx ON message (publish_date);

INSERT INTO message (id, channel_id, gallery_id, publish_date) SELECT id, channel_id, gallery_id, publish_date FROM _del_message;
DROP TABLE _del_message;
