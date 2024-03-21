-- Add up migration script here
DROP VIEW challenge_view;
CREATE VIEW challenge_view AS
SELECT gallery.id,
       gallery.token,
       JSON_EXTRACT(gallery.tags, '$.artist[0]') AS artist,
       page.page,
       image.id AS image_id,
       image.url,
       poll.score
FROM page
         LEFT JOIN gallery ON gallery.id = page.gallery_id
         LEFT JOIN image ON image.id = page.image_id
         LEFT JOIN poll ON poll.gallery_id = gallery.id
WHERE gallery.pages NOTNULL
    AND gallery.tags != ""
	AND JSON_ARRAY_LENGTH(JSON_EXTRACT(gallery.tags, '$.artist')) = 1;
