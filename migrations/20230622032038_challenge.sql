-- Add up migration script here

CREATE TABLE challenge_history (
	id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
	user_id INTEGER NOT NULL,
	gallery_id INTEGER NOT NULL,
	page INTEGER NOT NULL,
	success BOOLEAN NOT NULL,
	answer_time DATETIME NOT NULL
);

CREATE VIEW challenge_view AS
SELECT gallery.id, gallery.token, JSON_EXTRACT(gallery.tags, '$.artist[0]') AS artist, page.page, image.hash, image.url, poll.score FROM page
LEFT JOIN gallery ON  gallery.id = page.gallery_id
LEFT JOIN image ON image.id = page.image_id
LEFT JOIN poll ON poll.gallery_id = gallery.id
WHERE gallery.pages NOTNULL
	AND JSON_ARRAY_LENGTH(JSON_EXTRACT(gallery.tags, '$.artist')) = 1;

