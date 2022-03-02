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
	pub date: NaiveDateTime,
}

#[derive(Debug, Template, Default)]
#[template(path = "music_page.html")]
pub struct MusicPage {
	pub music: Vec<Music>,
}

pub async fn get_cache() -> DbResult<&'static RwLock<crate::CacheData<MusicPage>>> {
	static CACHE: Cache<MusicPage> = Cache::const_new();

	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await;

	let last = query!("SELECT music FROM cache")
		.fetch_one(db())
		.await?
		.music;

	if cache.read().await.time == last {
		return Ok(cache);
	}
	debug!("updating music cache");

	let music = query!(
		"SELECT music_id AS id, comment, title, date_uploaded AS date FROM music ORDER BY date DESC"
	)
	.fetch(db())
	.map_ok(|mut x| Music {
		id: x.id,
		title: x.title.take(),
		comment: x.comment.take(),
		media: Media::default(),
		date: x.date,
	})
	.try_collect::<Vec<_>>()
	.await?;

	let mut cached = cache.write().await;
	cached.data.music = music;
	cached.time = last;
	drop(cached);

	Ok(cache)
}

pub async fn handle_music(Path(id): Path<i32>) -> HttpResponse<Music> {
	async fn inner(id: i32) -> Result<Option<Music>> {
		let m = query!(
			"SELECT title, comment, file_path, date_uploaded FROM music WHERE music_id = $1",
			id
		)
		.fetch_optional(db())
		.await?
		.map(move |mut x| Music {
			id,
			title: x.title.take(),
			comment: x.comment.take(),
			date: x.date_uploaded,
			media: Media::new(x.file_path.take()),
		});

		Ok(m)
	}

	match inner(id).await {
		Ok(None) => Err(E404),
		Ok(Some(m)) => Ok(m),
		Err(e) => {
			error!("{e}");
			Err(E500)
		}
	}
}

pub async fn handle_music_page() -> HttpResponse<Html<String>> {
	match get_cache().await {
		Ok(c) => c.read().await.data.render().html().map_err(|e| e500!(e)),
		Err(e) => Err(e500!(e)),
	}
}
