use axum::extract::Path;

use crate::prelude::*;

pub struct ArticleInfo {
	pub title: String,
	pub url_title: String,
	pub published: DateTime<Utc>,
	pub updated: Option<DateTime<Utc>>,
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct Article {
	info: ArticleInfo,
	html: String,
}

pub async fn handle_article(Path(p): Path<String>) -> HtmlResponse<Article> {
	query!(
		"SELECT title, url_title, date_published, date_updated, html FROM ARTICLE WHERE url_title = $1",
		&p,
	)
	.fetch_optional(db())
	.await
	.or_500()
	.and_then(|opt| {
		opt.or_404().map(|mut x| Article {
			html: mem::take(&mut x.html),
			info: ArticleInfo {
				published: x.date_published,
				updated: x.date_updated,
				title: mem::take(&mut x.title),
				url_title: mem::take(&mut x.url_title),
			},
		})
	})
}
