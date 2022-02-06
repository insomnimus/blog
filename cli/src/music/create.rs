use crate::{
	prelude::*,
	sftp::SendFile,
};

pub fn app() -> App<'static> {
	App::new("create")
	.about("Create a new music post.")
	.args(&[
	arg!(-p --path <PATH> "Path to an audio file or `PATH::RENAME`.")
	.validator(super::validate_music),
	arg!(-c --comment [COMMENT] "A plaintext comment.")
	.validator(validate_comment),
	arg!(-r --sftp <URI> "The sftp server connection uri in the form `user@host:/path/to/storage`")
	.env("BLOG_SFTP_URI")
	.validator(validate_sftp_uri),
	arg!(title: [TITLE] "The title of the music post.")
	.validator(crate::article::validate_title),
	Arg::new("sftp-args")
	.multiple_values(true)
	.last(true)
	.help("Extra args to pass to the sftp command."),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let sftp = Config::sftp(m).await?;
	let title = m.value_of("title");
	let comment = m.value_of("comment");
	let media = m.value_of_t_or_exit::<SendFile>("path");

	let dir = format!("music_{}", rand_filename());
	let path = format!("{dir}/{remote}", remote = media.remote());
	sftp.send_files(&dir, &[media]).await?;

	println!("✓ uploaded file to the sftp server");

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
