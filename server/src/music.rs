use crate::{
	media::Media,
	prelude::*,
};

#[derive(Debug, Template)]
#[template(path = "music.html")]
pub struct Music {
	id: i32,
	title: Option<String>,
	comment: Option<String>,
	media: Media,
	date: String,
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
