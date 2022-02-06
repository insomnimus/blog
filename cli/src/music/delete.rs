use std::path::Path;

use crate::prelude::*;

pub fn app() -> App<'static> {
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
			arg!(-r --sftp [SFTP_URI] "The sftp server URI in the form `user@host:/path/to/store`")
				.validator(validate_sftp_uri)
				.required_unless_present("keep-media")
				.env("BLOG_SFTP_URI"),
			Arg::new("sftp-args")
				.last(true)
				.help("Extra arguments to pass to the sftp command.")
				.conflicts_with("keep-media")
				.multiple_values(true),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let yes = m.is_present("yes");
	let dirty = m.is_present("dirty");
	let keep_media = m.is_present("keep-media");
	let sftp = if keep_media {
		None
	} else {
		Some(Config::sftp(m).await?)
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

	if let Some(sftp) = sftp {
		let dirname = Path::new(&media).parent().unwrap().to_str().unwrap();
		match sftp.rmdir(dirname).await {
			Err(e) if dirty => eprintln!("warning: failed to delete the media: {e}"),
			Ok(_) => println!("✓ deleted uploaded media {media}"),
			res => res?,
		};
	}

	tx.commit().await?;

	println!("✓ deleted music {music}");
	Ok(())
}
