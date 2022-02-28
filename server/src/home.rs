use crate::{
	article::{
		self,
		ArticleInfo,
	},
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

async fn get_articles() -> DbResult<Vec<ArticleInfo>> {
	let cache = article::get_cache().await?.read().await;
	Ok(cache.data.articles.values().take(5).cloned().collect())
}

async fn get_posts() -> DbResult<Vec<PostInfo>> {
	query!(
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
	.fetch(db())
	.map_ok(|mut x| PostInfo {
		id: x.id,
		content: x.content.take(),
		n_attachments: x.n_attachments,
		date: x.date.format_utc(),
	})
	.try_collect()
	.await
}

async fn get_music() -> DbResult<Vec<Music>> {
	query!("SELECT music_id id, title, comment, date_uploaded date FROM music ORDER BY date DESC LIMIT 5")
	.fetch(db())
	.map_ok(|mut x| Music {
			id: x.id,
			date: x.date,
			title: x.title.take(),
			comment: x.comment.take(),
			media: Default::default(),
		})
		.try_collect()
		.await
}

pub async fn handle_home() -> HttpResponse {
	static CACHE: Cache = Cache::const_new();

	async fn inner() -> Result<Html<String>> {
		let cache = CACHE
			.get_or_init(|| async { RwLock::new(Default::default()) })
			.await;

		let last_updated = query!("SELECT home FROM cache").fetch_one(db()).await?.home;

		{
			let cached = cache.read().await;
			if cached.time == last_updated && !cached.data.is_empty() {
				return Ok(Html(cached.data.clone()));
			}
		}
		debug!("updating home cache");

		let articles = get_articles().await?;
		let posts = get_posts().await?;
		let music = get_music().await?;

		let home = Home {
			articles,
			posts,
			music,
		};
		let html = home.render()?;

		let mut cached = cache.write().await;
		cached.data.clear();
		cached.data.push_str(&html);
		cached.time = last_updated;

		Ok(Html(html))
	}

	inner().await.map_err(|e| e500!(e))
}
