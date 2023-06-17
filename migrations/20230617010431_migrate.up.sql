-- Add up migration script here
CREATE TABLE image (
    gallery_id INTEGER NOT NULL,
    page INTEGER NOT NULL,
    hash TEXT NOT NULL,
    url TEXT NOT NULL,
    PRIMARY KEY (gallery_id, page)
);
CREATE INDEX image_hash_index ON image (hash);

CREATE TABLE message (
    id INTEGER PRIMARY KEY NOT NULL,
    gallery_id INTEGER NOT NULL,
    telegraph TEXT NOT NULL,
    publish_date DATE NOT NULL
);
CREATE INDEX message_gallery_id_index ON message (gallery_id);
CREATE INDEX message_publish_date_index ON message (publish_date);

CREATE TABLE poll (
    id INTEGER PRIMARY KEY NOT NULL,
    gallery_id INTEGER NOT NULL,
    score FLOAT NOT NULL
);
CREATE INDEX poll_gallery_id_index ON poll (gallery_id);
CREATE INDEX poll_score_index ON poll (score);

CREATE TABLE vote (
    user_id INTEGER NOT NULL,
    poll_id INTEGER NOT NULL,
    option INTEGER NOT NULL,
    vote_time DATETIME NOT NULL,
    PRIMARY KEY (user_id, poll_id)
);
CREATE INDEX vote_poll_id_index ON vote (poll_id);

ALTER TABLE gallery RENAME TO ogallery;
CREATE TABLE gallery (
    id INTEGER PRIMARY KEY NOT NULL,
    token TEXT NOT NULL,
    title TEXT NOT NULL,
    tags TEXT NOT NULL,
    pages INTEGER NOT NULL,
    parent INTEGER,
    deleted BOOLEAN NOT NULL
);

-- NOTE: 此处通过暴力的字符串替换将 tuple[str, list[str]] 转换为了 dict[str, list[str]]
INSERT INTO gallery (id, token, title, tags, pages, parent, deleted)
SELECT gallery_id, token, title, REPLACE(REPLACE(REPLACE(REPLACE(tags, "[[", "{"), "]],[", "],"), ",[", ":["), "]]]", "]}"), upload_images, NULL, FALSE
FROM ogallery;
