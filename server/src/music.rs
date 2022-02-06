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
	pub fn short_comment(&'_ self, max: usize) -> Cow<'_, str> {
		match self
			.comment
			.as_deref()
			.and_then(|s| s.trim().split('\n').next())
		{
			None => "-".into(),
			Some(s) if s.len() <= max => s.into(),
			Some(s) => {
				let mut buf = String::with_capacity(max);
				for word in s.split_whitespace() {
					if buf.len() + 4 + word.len() >= max {
						buf.truncate(max - 3);
						buf.push_str("...");
						break;
					}
					buf.push(' ');
					buf.push_str(word);
				}
				buf.into()
			}
		}
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
	let last = query!("SELECT music FROM cache")
		.fetch_one(db())
		.await
		.or_500()?
		.music;

	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await;
	{
		let cached = cache.read().await;
		if cached.time == last && !cached.html.is_empty() {
			return Ok(Html(cached.html.clone()));
		}
	}
	info!("updating music cache");

	let mut stream = query!(
		"SELECT music_id id, comment, title, date_uploaded date FROM music ORDER BY date DESC"
	)
	.fetch(db());

	let mut music = Vec::with_capacity(16);
	while let Some(res) = stream.next().await {
		let mut x = res.or_500()?;
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
	cached.html.clear();
	cached.html.push_str(&html);
	cached.time = last;

	Ok(Html(html))
}
