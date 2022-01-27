use super::validate_post;
use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("create").about("Create a new post.").args(&[
		arg!(-s --syntax [SYNTAX] "The markup format of the source text.")
			.default_value("markdown")
			.possible_values(Syntax::VALUES)
			.ignore_case(true),
		arg!(content: <CONTENT> "The post content.").validator(validate_post),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let syntax = m.value_of_t_or_exit::<Syntax>("syntax");
	let raw = m.value_of("content").unwrap().trim();
	let content = syntax.render(raw);

	let mut tx = db().begin().await?;
	let id = query!(
		"INSERT INTO post(raw, content)
	VALUES($1, $2)
	RETURNING post_id",
		raw,
		&content
	)
	.fetch_one(&mut tx)
	.await?
	.post_id;

	clear!(posts).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ created new post (id = {id})");
	Ok(())
}
