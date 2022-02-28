pub mod index;

use axum::extract::Path;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct ArticleInfo {
	pub title: String,
	pub url_title: String,
	pub about: String,
	pub published: NaiveDateTime,
	pub updated: Option<NaiveDateTime>,
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
	prev: Option<ArticleInfo>,
	next: Option<ArticleInfo>,
}

pub async fn handle_article(Path(title): Path<String>) -> HttpResponse<Article> {
	let (prev, next) = index::get_adjacent(&title)
		.await
		.map_err(|e| e500!(e))?
		.or_404()?;

	query!(
		"SELECT a.title, a.url_title, a.about, a.date_published, a.date_updated, a.html,
		ARRAY_AGG(t.tag_name) tags_array
		FROM article a
		LEFT JOIN article_tag t
		ON a.article_id = t.article_id
		WHERE url_title = $1
		GROUP BY a.title, a.url_title",
		&title,
	)
	.fetch_optional(db())
	.await
	.map_err(|e| e500!(e))?
	.or_404()
	.map(move |mut x| Article {
		html: x.html.take(),
		next,
		prev,
		info: ArticleInfo {
			about: x.about.take(),
			published: x.date_published,
			updated: x.date_updated,
			title: x.title.take(),
			url_title: x.url_title.take(),
			tags: x.tags_array.take().unwrap_or_default(),
		},
	})
}

pub async fn handle_articles() -> HttpResponse<Html<String>> {
	static CACHE: Cache = OnceCell::const_new();

	async fn inner() -> Result<String> {
		let cache = CACHE
			.get_or_init(|| async { RwLock::new(Default::default()) })
			.await;

		let last_updated = query!("SELECT articles FROM cache")
			.fetch_one(db())
			.await?
			.articles;

		{
			let cached = cache.read().await;
			if cached.time == last_updated && !cached.data.is_empty() {
				return Ok(cached.data.clone());
			}
		}
		debug!("updating articles cache");

		let articles = query!(
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
		.fetch(db())
		.map_ok(|mut x| ArticleInfo {
			title: x.title.take(),
			url_title: x.url_title.take(),
			about: x.about.take(),
			updated: x.updated,
			published: x.published,
			tags: x.tags.take().into_iter().flatten().flatten().collect(),
		})
		.try_collect::<Vec<_>>()
		.await?;

		let html = ArticlesPage { articles }.render()?;

		let mut cached = cache.write().await;
		cached.data.clear();
		cached.data.push_str(&html);
		cached.time = last_updated;

		Ok(html)
	}

	inner().await.map(Html).map_err(|e| e500!(e))
}
