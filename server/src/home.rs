use crate::{
	article::ArticleInfo,
	post::PostInfo,
	prelude::*,
};

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
	articles: Vec<ArticleInfo>,
	posts: Vec<PostInfo>,
}

pub async fn handle_home() -> HttpResponse {
	static CACHE: OnceCell<RwLock<Cache>> = OnceCell::const_new();
	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Cache::default()) })
		.await;

	let last_updated = query!("SELECT home FROM cache")
		.fetch_one(db())
		.await
		.or_500()?
		.home;

	{
		let cached = cache.read().await;
		if cached.time == last_updated && !cached.html.is_empty() {
			return Ok(Html(cached.html.clone()));
		}
	}

	let articles = query!(
		"SELECT title,
			url_title,
			about,
			date_published AS published,
			date_updated AS updated
			FROM article
	ORDER BY COALESCE(date_updated, date_published) DESC
	LIMIT 5"
	)
	.fetch_all(db())
	.await
	.or_500()?
	.into_iter()
	.map(|mut x| ArticleInfo {
		title: x.title.take(),
		url_title: x.url_title.take(),
		about: x.about.take(),
		published: x.published.format_utc(),
		updated: x.updated.map(|d| d.format_utc()),
		tags: Vec::new(),
	})
	.collect::<Vec<_>>();

	let posts = query_as!(
		PostInfo,
		r#"SELECT p.post_id AS id,
		p.content,
		p.date_posted AS date,
		COALESCE(COUNT(m.file_path), 0) AS "n_attachments!"
	FROM post p
	LEFT JOIN post_media m
	ON m.post_id = p.post_id
	GROUP BY p.post_id
	ORDER BY p.post_id DESC
	LIMIT 10"#
	)
	.fetch_all(db())
	.await
	.or_500()?;

	let home = Home { articles, posts };
	let html = home.render().or_500()?;

	let mut cached = cache.write().await;
	cached.html.clear();
	cached.html.push_str(&html);
	cached.time = last_updated;

	Ok(Html(html))
}
