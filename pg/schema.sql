CREATE TABLE article (
	article_id SERIAL UNIQUE,
	url_title TEXT PRIMARY KEY NOT NULL,
	title TEXT NOT NULL,
	date_published TIMESTAMPTZ NOT NULL,
	date_updated TIMESTAMPTZ,
	html TEXT NOT NULL,
	markdown TEXT NOT NULL,
	markdown_hash BYTEA NOT NULL
);

CREATE TABLE home_cache (
	_home_id BOOL PRIMARY KEY DEFAULT TRUE,
	data TEXT NOT NULL,
	CONSTRAINT only_one_cache CHECK(_home_id)
);
