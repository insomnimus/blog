use crate::{
	article::ArticleInfo,
	music::Music,
	post::PostInfo,
	prelude::*,
};

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
	articles: Vec<ArticleInfo>,
	posts: Vec<PostInfo>,
	music: Vec<Music>,
}

async fn get_articles() -> anyhow::Result<Vec<ArticleInfo>> {
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
	.await?
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

	Ok(articles)
}

async fn get_posts() -> anyhow::Result<Vec<PostInfo>> {
	let posts = query!(
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
	.await?
	.into_iter()
	.map(|mut x| PostInfo {
		id: x.id,
		content: x.content.take(),
		n_attachments: x.n_attachments,
		date: x.date.format_utc(),
	})
	.collect::<Vec<_>>();

	Ok(posts)
}

async fn get_music() -> anyhow::Result<Vec<Music>> {
	let music = query!("SELECT music_id id, title, comment, date_uploaded date FROM music ORDER BY date DESC LIMIT 5")
	.fetch_all(db())
	.await?
	.into_iter()
	.map(|mut x| Music{
		id: x.id,
		date: x.date.format_utc(),
		title: x.title.take(),
		comment: x.comment.take(),
		media: Default::default(),
	});

	Ok(music.collect())
}

pub async fn handle_home() -> HttpResponse {
	static CACHE: Cache = Cache::const_new();

	let last_updated = query!("SELECT home FROM cache")
		.fetch_one(db())
		.await
		.or_500()?
		.home;

	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await;

	{
		let cached = cache.read().await;
		if cached.time == last_updated && !cached.html.is_empty() {
			return Ok(Html(cached.html.clone()));
		}
	}
	info!("updating home cache");

	let articles = get_articles().await.or_500()?;
	let posts = get_posts().await.or_500()?;
	let music = get_music().await.or_500()?;

	let home = Home {
		articles,
		posts,
		music,
	};
	let html = home.render().or_500()?;

	let mut cached = cache.write().await;
	cached.html.clear();
	cached.html.push_str(&html);
	cached.time = last_updated;

	Ok(Html(html))
}
