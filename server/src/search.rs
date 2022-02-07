use crate::{
	music::Music,
	article::ArticleInfo,
	prelude::*,
};

#[derive(Template)]
#[template(path = "search.html")]
pub struct SearchPage {
	is_base: bool,
	results: Vec<SearchResult>,
	title: String,
}

impl Default for SearchPage {
	fn default() -> Self {
		Self {
			is_base: true,
			results: Vec::new(),
			title: String::from("Search for posts"),
		}
	}
}

enum SearchResult {
	Article(ArticleInfo),
	Music(Music),
}

#[derive(Deserialize)]
pub struct SearchParams {
	kind: String,
	query: String,
}

pub async fn handle_search(params: Option<Query<SearchParams>>) -> HttpResponse<SearchPage> {
	let params = match params {
		None => return Ok(SearchPage::default()),
		Some(p) => p.0,
	};
	if params.query.len() >= 200 {
		return Err(E_BAD_REQUEST);
	}

	match params.kind.as_str() {
		"article" => search_article(params).await,
		"music" => search_music(params).await,
		_ => Err(E_BAD_REQUEST),
	}
}

async fn search_article(params: SearchParams) -> HttpResponse<SearchPage> {
	let mut tags = Vec::new();
	let mut term = String::new();
	for s in params.query.split_whitespace() {
		if let Some(tag) = s.strip_prefix('#') {
			if !tag.is_empty() {
				tags.push(tag.to_lowercase());
			}
		} else {
			if !term.is_empty() {
				term.push(' ');
			}
			term.push_str(s);
		}
	}

	if tags.is_empty() && term.is_empty() {
		return Ok(SearchPage::default());
	}

	let title = if term.is_empty() {
		String::new()
	} else {
		format!("%{}%", term.to_lowercase())
	};

	let results = query!(
		"SELECT
	a.title, a.url_title,
	a.date_published, a.date_updated,
	a.about, ARRAY_AGG(t.tag_name) tags_array
	FROM article a
	LEFT JOIN article_tag t
	ON t.article_id = a.article_id
	WHERE $1 = '' OR LOWER(a.title) LIKE $1
	GROUP BY a.title, a.url_title
	HAVING ARRAY_AGG(t.tag_name) @> $2
	ORDER BY COALESCE(a.date_updated, a.date_published) DESC",
		&title,
		&tags,
	)
	.fetch_all(db())
	.await
	.or_500()?
	.into_iter()
	.map(|mut x| {
		SearchResult::Article(ArticleInfo {
			title: x.title.take(),
			url_title: x.url_title.take(),
			published: x.date_published.format_utc(),
			updated: x.date_updated.map(|d| d.format_utc()),
			about: x.about.take(),
			tags: x.tags_array.take().unwrap_or_default(),
		})
	})
	.collect::<Vec<_>>();

	let title = if term.is_empty() {
		let mut buf = String::from("Articles tagged");
		for t in &tags {
			buf.push(' ');
			buf.push_str(t);
		}
		buf
	} else if tags.is_empty() {
		format!("Search results for '{}'", &term)
	} else {
		let mut buf = format!("Search results for '{}' tagged", &term);
		for t in &tags {
			buf.push(' ');
			buf.push_str(t);
		}
		buf
	};

	Ok(SearchPage {
		is_base: false,
		title,
		results,
	})
}

async fn search_music(params: SearchParams) -> HttpResponse<SearchPage> {
	let q = format!("%{}%", params.query.to_lowercase());
	let mut stream = query!("SELECT
	title,
	music_id AS id,
	date_uploaded AS date,
	comment
	FROM music
	WHERE LOWER(title) LIKE $1
	ORDER BY date DESC", q)
	.fetch(db());
	
	let mut results = Vec::new();
	while let Some(res) = stream.next().await {
		let mut x = res.or_500()?;
		results.push(SearchResult::Music(Music {
			id: x.id,
			date: x.date.format_utc(),
			title: x.title.take(),
			comment: x.comment.take(),
			media: Default::default(),
		}));
	}
	
	Ok(SearchPage {
		is_base: false,
		title: format!("Search Results for '{}'", &params.query),
		results,
	})
}