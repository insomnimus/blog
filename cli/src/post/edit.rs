use std::borrow::Cow;

use super::validate_post;
use crate::prelude::*;

pub fn app() -> App {
	App::new("edit")
		.about("Edit an existing post.")
		.group(ArgGroup::new("handle").required(true).args(&["id", "last"]))
		.args(&[
			arg!(-s --syntax [SYNTAX] "The markup format of the post.")
				.possible_values(Syntax::VALUES)
				.ignore_case(true),
			arg!(id: [ID] "The post id.")
				.validator(validate::<i32>("the value must be an integer")),
			arg!(--last "Edit the last post."),
			arg!(content: [CONTENT] "The new psot content. Omit to edit the psot interactively.")
				.validator(validate_post),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let (id, raw, syntax) = match m.value_of_t::<i32>("id") {
		Ok(id) => query!(
			r#"SELECT raw, syntax AS "syntax: Syntax" FROM post WHERE post_id = $1"#,
			id
		)
		.fetch_optional(db())
		.await?
		.map(|mut x| (id, x.raw.take(), x.syntax))
		.ok_or_else(|| anyhow!("no post found with the id {id}"))?,
		Err(_) => query!(
			r#"SELECT post_id, raw, syntax AS "syntax: Syntax" FROM post ORDER BY post_id DESC LIMIT 1"#
		)
		.fetch_optional(db())
		.await?
		.map(|mut x| (x.post_id, x.raw.take(), x.syntax))
		.ok_or_else(|| anyhow!("there are no posts in the database"))?,
	};

	let raw = match m.value_of("content") {
		Some(x) => Cow::Borrowed(x),
		None => match edit_buf("post", ".md", &raw).await? {
			None => return Ok(()),
			Some(x) => Cow::Owned(x),
		},
	};

	let syntax = m.value_of_t("syntax").unwrap_or(syntax);

	let content = syntax.render(&raw);

	let mut tx = db().begin().await?;

	query!(
		"UPDATE post SET raw = $1, content = $2,  syntax = $3 WHERE post_id = $4",
		&raw,
		&content,
		syntax as Syntax,
		id,
	)
	.execute(&mut tx)
	.await?;

	clear!(posts).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ updated post #{id}");
	Ok(())
}
