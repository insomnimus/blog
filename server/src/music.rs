use crate::{
	media::Media,
	prelude::*,
};

#[derive(Debug, Template)]
#[template(path = "music.html")]
pub struct Music {
	pub id: i32,
	pub title: Option<String>,
	pub comment: Option<String>,
	pub media: Media,
	pub date: String,
}

#[derive(Debug, Template)]
#[template(path = "music_page.html")]
pub struct MusicPage {
	music: Vec<Music>,
}

impl Music {
	pub fn short_comment(&'_ self, max: usize) -> Option<Cow<'_, str>> {
		self.comment.as_deref().map(|s| s.first_line_words(max))
	}
}

pub async fn handle_music(Path(id): Path<i32>) -> HttpResponse<Music> {
	query!(
		"SELECT title, comment, file_path, date_uploaded FROM music WHERE music_id = $1",
		id
	)
	.fetch_optional(db())
	.await
	.or_500()?
	.map(move |mut x| Music {
		id,
		title: x.title.take(),
		comment: x.comment.take(),
		date: x.date_uploaded.format_utc(),
		media: Media::new(x.file_path.take()),
	})
	.or_404()
}

pub async fn handle_music_page() -> HttpResponse {
	static CACHE: Cache = Cache::const_new();

	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await;

	let last = query!("SELECT music FROM cache")
		.fetch_one(db())
		.await
		.or_500()?
		.music;

	{
		let cached = cache.read().await;
		if cached.time == last && !cached.data.is_empty() {
			return Ok(Html(cached.data.clone()));
		}
	}
	info!("updating music cache");

	let mut stream = query!(
		"SELECT music_id id, comment, title, date_uploaded date FROM music ORDER BY date DESC"
	)
	.fetch(db());

	let mut music = Vec::with_capacity(16);
	while let Some(mut x) = stream.next().await.transpose().or_500()? {
		music.push(Music {
			id: x.id,
			title: x.title.take(),
			comment: x.comment.take(),
			media: Media::default(),
			date: x.date.format_utc(),
		});
	}

	let html = MusicPage { music }.render().or_500()?;
	let mut cached = cache.write().await;
	cached.data.clear();
	cached.data.push_str(&html);
	cached.time = last;

	Ok(Html(html))
}
