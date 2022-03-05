use crate::{
	media::Media,
	prelude::*,
};

#[derive(Template)]
#[template(path = "note.html")]
pub struct Note {
	id: i32,
	content: String,
	date: String,
	attachments: Vec<Media>,
}

pub struct NoteInfo {
	pub id: i32,
	pub content: String,
	pub date: NaiveDateTime,
	pub n_attachments: i64,
}

#[derive(Template)]
#[template(path = "note_page.html")]
pub struct NotePage {
	note: Note,
}

#[derive(Template)]
#[template(path = "notes_page.html")]
pub struct NotesPage {
	notes: Vec<Note>,
}

#[derive(Serialize)]
pub struct NotesJson {
	notes: Vec<String>,
}

#[derive(Deserialize)]
pub struct NoteParams {
	cursor: i32,
}

async fn get_notes(last_id: i32) -> DbResult<Vec<Note>> {
	query!(
		r#"SELECT
	n.note_id id,
	n.content,
	n.date_posted AS date,
	ARRAY_AGG(m.file_path) AS "attachments: Vec<Option<String>>"
	FROM note n
	LEFT JOIN note_media m
	ON n.note_id = m.note_id
	WHERE $1 = 1 OR n.note_id < $1
	GROUP BY n.note_id
	ORDER BY n.date_posted DESC
	LIMIT 50"#,
		last_id,
	)
	.fetch(db())
	.map_ok(|mut x| Note {
		id: x.id,
		content: x.content.take(),
		date: x.date.format_utc(),
		attachments: x
			.attachments
			.take()
			.into_iter()
			.flatten()
			.flatten()
			.map(Media::new)
			.collect(),
	})
	.try_collect()
	.await
}

pub async fn handle_notes() -> HttpResponse {
	static CACHE: Cache = Cache::const_new();

	async fn inner() -> Result<Html<String>> {
		let cache = CACHE
			.get_or_init(|| async { RwLock::new(Default::default()) })
			.await;

		let last_updated = query!("SELECT notes FROM cache")
			.fetch_one(db())
			.await?
			.notes;

		{
			let cached = cache.read().await;
			if cached.time == last_updated && !cached.data.is_empty() {
				return Ok(Html(cached.data.clone()));
			}
		}
		debug!("updating notes cache");

		let notes = get_notes(1).await?;
		let html = NotesPage { notes }.render()?;

		let mut cached = cache.write().await;
		cached.data.clear();
		cached.data.push_str(&html);
		cached.time = last_updated;

		Ok(Html(html))
	}

	inner().await.map_err(|e| e500!(e))
}

pub async fn handle_api(Query(params): Query<NoteParams>) -> HttpResponse<Json<NotesJson>> {
	get_notes(params.cursor)
		.await
		.map_err(|e| e500!(e))?
		.into_iter()
		.map(|n| n.render())
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| e500!(e))
		.map(|notes| Json(NotesJson { notes }))
}

pub async fn handle_note(Path(id): Path<i32>) -> HttpResponse<NotePage> {
	query!(
		r#"SELECT
	n.note_id AS id,
	n.content,
	n.date_posted AS date,
	ARRAY_AGG(m.file_path) AS "attachments?: Vec<Option<String>>"
	FROM note n
	LEFT JOIN note_media m
	ON n.note_id = m.note_id
	WHERE n.note_id = $1
	GROUP BY n.note_id"#,
		id
	)
	.fetch_optional(db())
	.await
	.map_err(|e| e500!(e))?
	.or_404()
	.map(|mut x| NotePage {
		note: Note {
			id: x.id,
			content: x.content.take(),
			date: x.date.format_utc(),
			attachments: x
				.attachments
				.take()
				.into_iter()
				.flatten()
				.flatten()
				.map(Media::new)
				.collect(),
		},
	})
}
