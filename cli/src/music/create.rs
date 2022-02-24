use crate::{
	media::{
		self,
		SendFile,
	},
	prelude::*,
};

pub fn app() -> App {
	App::new("create").about("Create a new music post.").args(&[
		arg!(-p --path <PATH> "Path to an audio file or `PATH::RENAME`.")
			.validator(super::validate_music),
		arg!(-c --comment [COMMENT] "A plaintext comment.").validator(validate_comment),
		arg!(title: [TITLE] "The title of the music post.")
			.validator(crate::article::validate_title),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let root = Config::media_dir(m).await?;
	let title = m.value_of("title");
	let comment = m.value_of("comment");
	let media = m.value_of_t_or_exit::<SendFile>("path");

	let dir = rand_filename("music_");
	let path = format!("{dir}/{remote}", remote = media.remote());
	run_hook!(pre_media, m).await?;
	media::send_files(&root.join(&dir), &[media]).await?;

	println!("✓ copied the file to the media directory");

	let mut tx = db().begin().await?;

	query!("INSERT INTO media(file_path) VALUES($1)", &path)
		.execute(&mut tx)
		.await?;

	let id = query!(
		"INSERT INTO music(title, comment, file_path)
	VALUES($1, $2, $3)
	RETURNING music_id",
		title,
		comment,
		&path,
	)
	.fetch_one(&mut tx)
	.await?
	.music_id;

	clear!(music).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ created a new music post (id = {id}, attachment = {path})");

	std::env::set_var("BLOG_CREATED", &dir);
	run_hook!(post_media, m)
		.await
		.context("failed to run the post-media hook but the operation was successful")?;

	Ok(())
}

fn validate_comment(s: &str) -> StdResult<(), String> {
	match s.trim().chars().count() {
		0..=720 => Ok(()),
		_ => Err(String::from(
			"the value is too long; maximum character limit is 720",
		)),
	}
}
