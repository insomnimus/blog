use std::{
	borrow::Cow,
	mem,
};

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
	query!(
		"SELECT title, date_published, date_updated, data FROM ARTICLE WHERE title = $1",
		&p,
	)
	.fetch_optional(db())
	.await
	.or_500()
	.and_then(|opt| {
		opt.or_404().map(|mut x| Article {
			html: mem::take(&mut x.data),
			info: ArticleInfo {
				published: x.date_published,
				updated: x.date_updated,
				title: mem::take(&mut x.title),
			},
		})
	})
}
