CREATE TYPE syntax AS ENUM('plain', 'markdown', 'html');

CREATE TABLE cache(
	_instance BOOL PRIMARY KEY NOT NULL DEFAULT TRUE,
	home TIMESTAMP NOT NULL DEFAULT(CURRENT_TIMESTAMP),
	articles TIMESTAMP NOT NULL DEFAULT(CURRENT_TIMESTAMP),
	posts TIMESTAMP NOT NULL DEFAULT(CURRENT_TIMESTAMP),
	CONSTRAINT only_one_cache CHECK(_instance)
);

CREATE TABLE media (
	media_id SERIAL NOT NULL,
	file_path TEXT NOT NULL PRIMARY KEY,
	comment TEXT
);

CREATE TABLE article (
	article_id SERIAL UNIQUE NOT NULL,
	url_title TEXT PRIMARY KEY NOT NULL,
	title TEXT NOT NULL,
	about TEXT NOT NULL,
	date_published TIMESTAMP NOT NULL DEFAULT(NOW() AT TIME ZONE 'UTC'),
	date_updated TIMESTAMP,
	html TEXT NOT NULL,
	raw TEXT NOT NULL,
	raw_hash BYTEA NOT NULL,
	syntax syntax NOT NULL
);

CREATE TABLE tag(
	tag_name TEXT PRIMARY KEY NOT NULL
);

CREATE TABLE article_tag(
	entry_id SERIAL PRIMARY KEY NOT NULL,
	article_id INT NOT NULL REFERENCES article(article_id) ON DELETE CASCADE ON UPDATE CASCADE,
	tag_name TEXT NOT NULL REFERENCES tag(tag_name) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE post (
	post_id SERIAL PRIMARY KEY NOT NULL,
	raw TEXT NOT NULL,
	content TEXT NOT NULL,
	date_posted TIMESTAMP NOT NULL DEFAULT(NOW() AT TIME ZONE 'UTC')
);

CREATE TABLE post_media (
	file_path TEXT NOT NULL REFERENCES media ON DELETE CASCADE ON UPDATE CASCADE,
	post_id INTEGER NOT NULL REFERENCES post ON DELETE CASCADE ON UPDATE CASCADE
);

-- Initialization --
INSERT INTO cache(_instance, home, articles, posts)
VALUES('TRUE', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
