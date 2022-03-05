BEGIN;

ALTER TABLE cache
RENAME COLUMN posts TO notes;

ALTER TABLE post_media
RENAME COLUMN post_id TO note_id;

ALTER TABLE post
RENAME COLUMN post_id TO note_id;

ALTER TABLE post_media RENAME TO note_media;
ALTER TABLE post RENAME TO note;

COMMIT;
