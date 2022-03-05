use std::borrow::Cow;

use super::validate_post;
use crate::prelude::*;

pub fn app() -> App {
	App::new("edit")
		.about("Edit a note.")
		.group(ArgGroup::new("handle").required(true).args(&["id", "last"]))
		.args(&[
			arg!(-s --syntax [SYNTAX] "The markup format of the post.")
				.possible_values(Syntax::VALUES)
				.ignore_case(true),
			arg!(id: [ID] "The note id.")
				.validator(validate::<i32>("the value must be an integer")),
			arg!(--last "Edit the last note."),
			arg!(content: [CONTENT] "The new psot content. Omit to edit the psot interactively.")
				.validator(validate_post),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let (id, raw, syntax) = match m.value_of_t::<i32>("id") {
		Ok(id) => query!(
			r#"SELECT raw, syntax AS "syntax: Syntax" FROM note WHERE note_id = $1"#,
			id
		)
		.fetch_optional(db())
		.await?
		.map(|mut x| (id, x.raw.take(), x.syntax))
		.ok_or_else(|| anyhow!("no note found with the id {id}"))?,
		Err(_) => query!(
			r#"SELECT note_id AS id, raw, syntax AS "syntax: Syntax" FROM note ORDER BY id DESC LIMIT 1"#
		)
		.fetch_optional(db())
		.await?
		.map(|mut x| (x.id, x.raw.take(), x.syntax))
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
		"UPDATE note SET raw = $1, content = $2,  syntax = $3 WHERE note_id = $4",
		&raw,
		&content,
		syntax as Syntax,
		id,
	)
	.execute(&mut tx)
	.await?;

	clear!(notes).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ updated note #{id}");
	Ok(())
}
