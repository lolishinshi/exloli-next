-- Add up migration script here
ALTER TABLE poll RENAME TO _del_poll;
DROP INDEX poll_gallery_id_idx;
DROP INDEX poll_score_idx;

CREATE TABLE poll (
    id INTEGER NOT NULL,
    gallery_id INTEGER NOT NULL,
    score FLOAT NOT NULL,
    PRIMARY KEY (id, gallery_id)
);
CREATE INDEX poll_gallery_id_idx ON poll (gallery_id);
CREATE INDEX poll_score_idx ON poll (score);

INSERT INTO poll (id, gallery_id, score) SELECT id, gallery_id, score FROM _del_poll;
DROP TABLE _del_poll;

UPDATE poll
SET gallery_id = (
    SELECT message.gallery_id
    FROM message
    WHERE message.id = poll.gallery_id
)
WHERE id = gallery_id;
