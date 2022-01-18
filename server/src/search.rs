use crate::{
	article::ArticleInfo,
	prelude::*,
};

#[derive(Template)]
#[template(path = "search.html")]]
struct Search {
	results: Vec<ArticleInfo>,
	query: SearchParams,
}

impl Search {
	fn title(&self) -> String {
		match &self.query.query {
			Some(q) if self.query.tags.is_empty() => format!("Search results for '{}'", q),
			Some(q) => {
				let mut buf = format!("Search results for '{}' tagged", q);
				for tag in &self.query.tags {
					buf.push(' ');
					buf.push_str(tag);
				}
				buf
			}
			None if self.query.tags.is_empty() => "Search for articles".into(),
			None => {
				let mut buf = String::from("Articles tagged");
				for tag in self.query.tags {
					buf.push(' ');
					buf.push_str(tag);
				}
				buf
			}
		}
	}
}

#[derive(Deserialize)]
pub struct SearchParams {
	#[serde(default)]
	query: Option<String>,
	#[serde(default)]
	tags: Vec<String>,
}

pub async fn handle_search(
	Query(params): Query<SearchParams>,
) -> HttpResponse<Search>{
	let SearchParams{query, tags} = params;
	let q = query.as_ref().map(|s| format!("%{}%", s.to_lowercase())).unwrap_or_default();

	let vals = query!(
		"SELECT
		a.title,
		a.url_title,
		a.date_published AS published,
		a.date_updated AS updated,
		ARRAY_AGG(t.tag_name) AS tags
		FROM article
		LEFT JOIN article_tag t
		ON a.article_id = t.article_id
	WHERE $1 = '' OR LOWER(title) LIKE $1
	GROUP BY a.title, a.url_title
	HAVING ARRAY_AGG(t.tag_name) @> $2
	ORDER BY COALESCE(date_updated, date_published) DESC",
		&q,
	)
	.fetch_all(db())
	.await
	.or_500()
	.map(|v| {
		v.into_iter()
			.map(|mut x| ArticleInfo {
				title: x.title.take(),
				url_title: x.url_title.take(),
				published: x.published.format_utc(),
				updated: x.updated.map(|d| d.format_utc()),
			})
			.collect()
	})
	.map(Json)
}
