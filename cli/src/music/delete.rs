use crate::{
	media,
	prelude::*,
};

pub fn app() -> App {
	App::new("delete")
		.about("Delete posted music.")
		.group(ArgGroup::new("handle").required(true).args(&["id", "last"]))
		.args(&[
			arg!(-l --last "Delete the last published music."),
			arg!(id: [ID] "The ID of the music.")
				.validator(validate::<i32>("the value must be a positive integer")),
			arg!(-y --yes "Do not prompt for confirmation."),
			arg!(--dirty "Allow failure to delete the uploaded media, delete the entry from the database."),
			arg!(--"keep-media" "Do not delete the uploaded media; just delete the post.")
				.conflicts_with("dirty"),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let yes = m.is_present("yes");
	let dirty = m.is_present("dirty");
	let keep_media = m.is_present("keep-media");
	let root = if keep_media {
		None
	} else {
		Some(Config::media_dir(m).await?)
	};

	let (id, title, media) = match m.value_of_t::<i32>("id") {
		Ok(id) => query!("SELECT title, file_path FROM music WHERE music_id = $1", id)
			.fetch_optional(db())
			.await?
			.map(|mut x| (id, x.title.take(), x.file_path.take()))
			.ok_or_else(|| anyhow!("no music found with the id {id}"))?,
		Err(_) => {
			query!("SELECT music_id, title, file_path FROM music ORDER BY music_id DESC LIMIT 1")
				.fetch_optional(db())
				.await?
				.map(|mut x| (x.music_id, x.title.take(), x.file_path.take()))
				.ok_or_else(|| anyhow!("there are no music posts in the database"))?
		}
	};
	let music = title.unwrap_or_else(|| format!("#{id}"));

	if !yes && !confirm!("delete music {music} ({media})?")? {
		println!("aborted");
		return Ok(());
	}

	let mut tx = db().begin().await?;
	query!("DELETE FROM music WHERE music_id = $1", id)
		.execute(&mut tx)
		.await?;
	query!("DELETE FROM media WHERE file_path = $1", &media)
		.execute(&mut tx)
		.await?;

	if let Some(root) = &root {
		run_hook!(pre_media, m).await?;
		match media::remove_files(root, &[&media]).await {
			Err(e) if dirty => {
				eprintln!("warning: failed to delete the file: {e}");
			}
			Ok(_) => {
				println!("✓ deleted the file from the media directory:  {media}");
			}
			Err(e) => return Err(e.into()),
		}
	}

	clear!(music).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ deleted music {music}");

	Ok(())
}
