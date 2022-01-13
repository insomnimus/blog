use std::borrow::Cow;

use axum::extract::Path;

use crate::prelude::*;

pub struct ArticleInfo {
	pub title: String,
	pub published: DateTime<Utc>,
	pub updated: Option<DateTime<Utc>>,
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct Article {
	info: ArticleInfo,
	html: String,
}

impl ArticleInfo {
	pub fn url_title(&'_ self) -> Cow<'_, str> {
		url_escape::encode_component(&self.title)
	}
}

pub async fn handle_article(Path(p): Path<String>) -> HtmlResponse<Article> {
	let p = url_escape::decode(&p);
	query_c!(
		"SELECT title, date_published, date_updated, data FROM ARTICLE WHERE title = $1",
		&p,
	)
	.map(|row| Article {
		html: row.get("html"),
		info: ArticleInfo {
			title: row.get("title"),
			published: row.get("date_published"),
			updated: row.get("date_updated"),
		},
	})
	.fetch_optional(db())
	.await
	.or_500()
	.and_then(|opt| opt.or_404())
}
