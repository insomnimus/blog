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
#[template(path = "articles_page.html")]
pub struct ArticlesPage {
	articles: Vec<ArticleInfo>,
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

pub async fn handle_articles() -> HttpResponse<Html<String>> {
	static CACHE: OnceCell<RwLock<Cache>> = OnceCell::const_new();
	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Cache::default()) })
		.await;

	let last_updated = query!("SELECT articles FROM cache")
		.fetch_one(db())
		.await
		.or_500()?
		.articles;

	{
		let cached = cache.read().await;
		if cached.time == last_updated && !cached.html.is_empty() {
			return Ok(Html(cached.html.clone()));
		}
	}
	info!("updating articles cache");

	let mut stream = query!(
		r#"SELECT
	a.url_title,
	a.title,
	a.about,
	a.date_published AS published,
	a.date_updated AS updated,
	ARRAY_AGG(t.tag_name) AS "tags?: Vec<Option<String>>"
	FROM article a
	LEFT JOIN article_tag t
	ON a.article_id = t.article_id
	GROUP BY a.article_id, a.url_title, a.title
	ORDER BY COALESCE(a.date_updated, a.date_published) DESC"#
	)
	.fetch(db());

	let mut articles = Vec::new();

	while let Some(res) = stream.next().await {
		let mut x = res.or_500()?;
		articles.push(ArticleInfo {
			title: x.title.take(),
			url_title: x.url_title.take(),
			about: x.about.take(),
			updated: x.updated.format_utc(),
			published: x.published.format_utc(),
			tags: x.tags.take().into_iter().flatten().flatten().collect(),
		});
	}

	let html = ArticlesPage { articles }.render().or_500()?;

	let mut cached = cache.write().await;
	cached.html.clear();
	cached.html.push_str(&html);
	cached.time = last_updated;

	Ok(Html(html))
}
