-- Add up migration script here
CREATE TABLE page (
    gallery_id INTEGER NOT NULL,
    page INTEGER NOT NULL,
    image_id INTEGER NOT NULL,
    PRIMARY KEY (gallery_id, page)
);
CREATE INDEX page_gallery_id_idx ON page (gallery_id);

CREATE TABLE image (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    hash TEXT UNIQUE NOT NULL,
    url TEXT NOT NULL
);
CREATE INDEX image_hash_idx ON image (hash);

CREATE TABLE message (
    id INTEGER PRIMARY KEY NOT NULL,
    gallery_id INTEGER NOT NULL,
    telegraph TEXT NOT NULL,
    publish_date DATE NOT NULL
);
CREATE INDEX message_gallery_id_idx ON message (gallery_id);
CREATE INDEX message_publish_date_idx ON message (publish_date);

CREATE TABLE poll (
    id INTEGER PRIMARY KEY NOT NULL,
    gallery_id INTEGER NOT NULL,
    score FLOAT NOT NULL
);
CREATE INDEX poll_gallery_id_idx ON poll (gallery_id);
CREATE INDEX poll_score_idx ON poll (score);

CREATE TABLE vote (
    user_id INTEGER NOT NULL,
    poll_id INTEGER NOT NULL,
    option INTEGER NOT NULL,
    vote_time DATETIME NOT NULL,
    PRIMARY KEY (user_id, poll_id)
);
CREATE INDEX vote_poll_id_idx ON vote (poll_id);

ALTER TABLE gallery RENAME TO ogallery;
CREATE TABLE gallery (
    id INTEGER PRIMARY KEY NOT NULL,
    token TEXT NOT NULL,
    title TEXT NOT NULL,
    title_jp TEXT,
    tags TEXT NOT NULL,
    pages INTEGER NOT NULL,
    parent INTEGER,
    deleted BOOLEAN NOT NULL
);

-- NOTE: 此处通过暴力的字符串替换将 tuple[str, list[str]] 转换为了 dict[str, list[str]]
INSERT INTO gallery (id, token, title, tags, pages, parent, deleted)
SELECT gallery_id, token, title, REPLACE(REPLACE(REPLACE(REPLACE(tags, "[[", "{"), "]],[", "],"), ",[", ":["), "]]]", "]}"), upload_images, NULL, FALSE
FROM ogallery
GROUP BY gallery_id;

INSERT INTO vote (user_id, poll_id, option, vote_time) SELECT user_id, poll_id, option, vote_time FROM user_vote;
INSERT INTO poll (id, gallery_id, score) SELECT CAST(poll_id AS INTEGER), gallery_id, score FROM ogallery GROUP BY poll_id;
INSERT INTO message (id, gallery_id, telegraph, publish_date) SELECT message_id, gallery_id, telegraph, publish_date FROM ogallery;
INSERT INTO image (hash, url) SELECT hash, url FROM image_hash;

DROP INDEX IF EXISTS  gallery_id_index;
DROP INDEX IF EXISTS  poll_id_index;
DROP INDEX IF EXISTS  poll_index;
DROP TABLE IF EXISTS  ogallery;
DROP TABLE IF EXISTS  user_vote;
DROP TABLE IF EXISTS  image_hash;
DROP TABLE IF EXISTS  images;
DROP TABLE IF EXISTS __diesel_schema_migrations;
