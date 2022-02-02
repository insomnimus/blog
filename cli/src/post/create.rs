use std::borrow::Cow;

use super::validate_post;
use crate::{
	prelude::*,
	sftp::SendFile,
};

pub fn app() -> App<'static> {
	App::new("create").about("Create a new post.").args(&[
		arg!(-s --syntax [SYNTAX] "The markup format of the source text.")
			.default_value("markdown")
			.possible_values(Syntax::VALUES)
			.ignore_case(true),
			arg!(-a --attachment [ATTACHMENT] ... "An attachment as a file path or `file::rename_name`.")
			.max_occurrences(3)
			.validator(validate_send_file)
			.requires("sftp"),
			arg!(-r --sftp [URI] "The sftp servers connection uri in the form `user@domain:/path/to/store`.")
			.env("BLOG_SFTP_URI")
			.validator(validate_sftp_uri),
		arg!(content: [CONTENT] "The post content. Omit to use your editor.").validator(validate_post),
		Arg::new("sftp-args")
		.multiple_values(true)
		.last(true)
		.help("Extra args to pass to the sftp command.")
		.required(false)
		.requires("sftp"),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let syntax = m.value_of_t_or_exit::<Syntax>("syntax");
	let raw = match m.value_of("content") {
		Some(s) => Cow::Borrowed(s.trim()),
		None => match edit_buf("new_post_", syntax.ext(), "").await? {
			None => {
				println!("cancelled");
				return Ok(());
			}
			Some(buf) => {
				if let Err(e) = validate_post(&buf) {
					return Err(anyhow!("post body is incorrect: {e}"));
				}
				Cow::Owned(buf)
			}
		},
	};

	let content = syntax.render(&raw);

	let mut tx = db().begin().await?;
	let id = query!(
		"INSERT INTO post(raw, content)
	VALUES($1, $2)
	RETURNING post_id",
		&raw,
		&content
	)
	.fetch_one(&mut tx)
	.await?
	.post_id;

	if let Ok(files) = m.values_of_t::<SendFile>("attachment") {
		let dir = format!("post_{id}");
		let sftp = sftp_args(m);
		sftp.send_files(&dir, &files).await?;
		for f in &files {
			let path = format!("{dir}/{remote}", remote = f.remote());

			query!("INSERT INTO media(file_path) VALUES($1)", &path,)
				.execute(&mut tx)
				.await?;

			query!(
				"INSERT INTO post_media(file_path, post_id) VALUES($1, $2)",
				&path,
				id,
			)
			.execute(&mut tx)
			.await?;
			println!(
				"✓ inserted attachment info for {} to the database",
				f.remote(),
			);
		}
	}

	clear!(posts).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ created new post (id = {id})");
	Ok(())
}

fn validate_send_file(s: &str) -> StdResult<(), String> {
	s.parse::<SendFile>()
		.map(|_| {})
		.map_err(|e| format!("error validating file: {e}"))
}
