use std::borrow::Cow;

use super::validate_post;
use crate::{
	media::{
		self,
		SendFile,
	},
	prelude::*,
};

pub fn app() -> App {
	App::new("create").about("Create a new post.").args(&[
		arg!(-s --syntax [SYNTAX] "The markup format of the source text.")
			.default_value("markdown")
			.possible_values(Syntax::VALUES)
			.ignore_case(true),
			arg!(-a --attachment [ATTACHMENT] ... "An attachment as a file path or `file::rename_name`.")
			.max_occurrences(3)
			.validator(validate_send_file),
		arg!(content: [CONTENT] "The post content. Omit to use your editor.").validator(validate_post),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let files = m.values_of_t::<SendFile>("attachment").ok();
	let dir = rand_filename("post_");
	if let Some(files) = &files {
		let root = Config::media_dir(m).await?;
		run_hook!(pre_media, m).await?;
		media::send_files(&root.join(&dir), files).await?;
	}

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
		"INSERT INTO post(raw, content, syntax)
	VALUES($1, $2, $3)
	RETURNING post_id",
		&raw,
		&content,
		syntax as Syntax,
	)
	.fetch_one(&mut tx)
	.await?
	.post_id;

	if let Some(files) = &files {
		for f in files {
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
