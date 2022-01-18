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

CREATE TABLE home_cache (
	_home_id BOOL PRIMARY KEY DEFAULT TRUE,
	data TEXT NOT NULL,
	CONSTRAINT only_one_cache CHECK(_home_id)
);

CREATE TABLE tag(
	tag_name TEXT PRIMARY KEY
);

CREATE TABLE article_tag(
	entry_id SERIAL PRIMARY KEY,
	article_id INT NOT NULL REFERENCES article(article_id) ON DELETE CASCADE ON UPDATE CASCADE,
	tag_name TEXT NOT NULL REFERENCES tag(tag_name) ON DELETE CASCADE ON UPDATE CASCADE
);
