use crate::{
	article::{
		self,
		ArticleInfo,
	},
	music::Music,
	note::NoteInfo,
	prelude::*,
};

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
	articles: Vec<ArticleInfo>,
	notes: Vec<NoteInfo>,
	music: Vec<Music>,
}

async fn get_articles() -> DbResult<Vec<ArticleInfo>> {
	let cache = article::get_cache().await?.read().await;
	Ok(cache.data.articles.values().take(5).cloned().collect())
}

async fn get_notes() -> DbResult<Vec<NoteInfo>> {
	query!(
		r#"SELECT n.note_id AS id,
		n.content,
		n.date_posted AS date,
		COALESCE(COUNT(m.file_path), 0) AS "n_attachments!"
	FROM note n
	LEFT JOIN note_media m
	ON n.note_id = m.note_id
	GROUP BY n.note_id
	ORDER BY n.note_id DESC
	LIMIT 10"#
	)
	.fetch(db())
	.map_ok(|mut x| NoteInfo {
		id: x.id,
		content: x.content.take(),
		n_attachments: x.n_attachments,
		date: x.date,
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
		let notes = get_notes().await?;
		let music = get_music().await?;

		let home = Home {
			articles,
			notes,
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
