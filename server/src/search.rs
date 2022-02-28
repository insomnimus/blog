use crate::{
	article::{
		self,
		ArticleInfo,
	},
	music::Music,
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
		return Err(E400);
	}

	match params.kind.as_str() {
		"article" => search_article(params).await,
		"music" => search_music(params).await.map_err(|e| e500!(e)),
		_ => Err(E400),
	}
}

async fn search_article(params: SearchParams) -> HttpResponse<SearchPage> {
	let mut tags = Vec::new();
	let mut term = String::new();
	for s in params.query.split_whitespace() {
		if let Some(tag) = s.strip_prefix('#') {
			if !tag.is_empty() {
				tags.push(tag);
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
		term.to_lowercase()
	};

	let results = article::get_cache()
		.await
		.map_err(|e| e500!(e))?
		.read()
		.await
		.data
		.articles
		.values()
		.filter(|a| title.is_empty() || a.title.to_lowercase().contains(&title))
		.filter(|a| is_subset(&tags, &a.tags, |a, b| a.eq_ignore_ascii_case(b)))
		.map(|a| SearchResult::Article(a.clone()))
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

async fn search_music(params: SearchParams) -> DbResult<SearchPage> {
	let q = format!("%{}%", params.query.to_lowercase());
	let results = query!(
		"SELECT
	title,
	music_id AS id,
	date_uploaded AS date,
	comment
	FROM music
	WHERE LOWER(title) LIKE $1
	ORDER BY date DESC",
		q
	)
	.fetch(db())
	.map_ok(|mut x| {
		SearchResult::Music(Music {
			id: x.id,
			date: x.date,
			title: x.title.take(),
			comment: x.comment.take(),
			media: Default::default(),
		})
	})
	.try_collect::<Vec<_>>()
	.await?;

	Ok(SearchPage {
		is_base: false,
		title: format!("Search Results for '{}'", &params.query),
		results,
	})
}

#[inline]
fn is_subset<'a, Set, Sub, F>(sub: &'a [Sub], set: &'a [Set], mut comp: F) -> bool
where
	F: FnMut(&'a Sub, &'a Set) -> bool,
{
	set.len() >= sub.len() && sub.iter().all(|s| set.iter().any(|x| comp(s, x)))
}

#[cfg(test)]
#[test]
fn test_subset() {
	const TESTS: &[(&[i32], &[i32])] = &[
		(&[1, 2, 3, 4, 5, 6], &[2, 4]),
		(&[5, 10, 2, 30], &[30]),
		(&[2], &[2]),
		(&[5, 4, 3, 2], &[2, 3, 4]),
	];

	for (set, sub) in TESTS {
		assert!(
			is_subset(sub, set, |a, b| a == b),
			"expected to be supset:\nset: {set:?}\nsub: {sub:?}"
		);
	}

	const FAIL: &[(&[&str], &[&str])] = &[
		(&["a", "b", "c", "d"], &[""]),
		(&["hey", "jude"], &["j"]),
		(&["lol"], &["lol", "bar"]),
	];

	for (set, sub) in FAIL {
		assert!(
			!is_subset(sub, set, |a, b| a.eq_ignore_ascii_case(b)),
			"not supposed to be subset but is:\nset: {set:?}\nsub: {sub:?}",
		);
	}
}
