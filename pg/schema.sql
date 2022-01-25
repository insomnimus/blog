CREATE TABLE home_cache (
	_home_id BOOL PRIMARY KEY DEFAULT TRUE,
	data TEXT NOT NULL,
	CONSTRAINT only_one_cache CHECK(_home_id)
);

CREATE TABLE media (
	media_id SERIAL,
	file_path TEXT NOT NULL PRIMARY KEY,
	comment TEXT
);

CREATE TABLE article (
	article_id SERIAL UNIQUE,
	url_title TEXT PRIMARY KEY NOT NULL,
	title TEXT NOT NULL,
	about TEXT NOT NULL,
	date_published TIMESTAMP NOT NULL DEFAULT(NOW() AT TIME ZONE 'UTC'),
	date_updated TIMESTAMP,
	html TEXT NOT NULL,
	markdown TEXT NOT NULL,
	markdown_hash BYTEA NOT NULL
);

CREATE TABLE tag(
	tag_name TEXT PRIMARY KEY
);

CREATE TABLE article_tag(
	entry_id SERIAL PRIMARY KEY,
	article_id INT NOT NULL REFERENCES article(article_id) ON DELETE CASCADE ON UPDATE CASCADE,
	tag_name TEXT NOT NULL REFERENCES tag(tag_name) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE post (
	post_id SERIAL PRIMARY KEY,
	markdown TEXT,
	html TEXT,
	date_posted TIMESTAMP NOT NULL DEFAULT(NOW() AT TIME ZONE 'UTC')
);

CREATE TABLE post_media (
	file_path TEXT REFERENCES media ON DELETE CASCADE ON UPDATE CASCADE,
	post_id INTEGER REFERENCES post ON DELETE CASCADE ON UPDATE CASCADE
);
