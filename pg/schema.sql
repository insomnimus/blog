CREATE TABLE article (
	article_id SERIAL UNIQUE,
	title TEXT PRIMARY KEY NOT NULL,
	date_published TIMESTAMPTZ NOT NULL,
	date_updated TIMESTAMPTZ,
	data TEXT NOT NULL
);

CREATE TABLE home_cache (
	_home_id BOOL PRIMARY KEY DEFAULT TRUE,
	data TEXT NOT NULL,
	CONSTRAINT only_one_cache CHECK(_home_id)
);
