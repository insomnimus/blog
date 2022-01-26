use axum::extract::Path;

use crate::prelude::*;

#[derive(Serialize)]
pub struct ArticleInfo {
	pub title: String,
	pub url_title: String,
	pub about: String,
	pub published: String,
	pub updated: Option<String>,
	pub tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct Article {
	info: ArticleInfo,
	html: String,
}

pub async fn handle_article(Path(title): Path<String>) -> HttpResponse<Article> {
	query!(
		"SELECT a.title, a.url_title, a.about, a.date_published, a.date_updated, a.html,
		ARRAY_AGG(t.tag_name) tags_array
		FROM article a
		LEFT JOIN article_tag t
		ON a.article_id = t.article_id
		WHERE url_title = $1
		GROUP BY a.title, a.url_title
		LIMIT 1",
		&title,
	)
	.fetch_optional(db())
	.await
	.or_500()
	.and_then(|opt| {
		opt.or_404().map(|mut x| Article {
			html: x.html.take(),
			info: ArticleInfo {
				about: x.about.take(),
				published: x.date_published.format_utc(),
				updated: x.date_updated.format_utc(),
				title: x.title.take(),
				url_title: x.url_title.take(),
				tags: x.tags_array.take().unwrap_or_default(),
			},
		})
	})
}
