use super::{
	validate_about,
	validate_tag,
	validate_title,
	ArticleContents,
};
use crate::prelude::*;

pub fn app() -> App {
	App::new("create").about("Publish a new article.").args(&[
		arg!(-p --path <FILE> "The article."),
		arg!(title: <TITLE> "The articles title.").validator(validate_title),
		arg!(-a --about <DESCRIPTION> "The article description.")
			.validator(validate_about)
			.visible_alias("description"),
		arg!(--"no-tags" "Permit omitting any tag.").conflicts_with("tags"),
		arg!(-s --syntax [SYNTAX] "The syntax for rendering. If omitted, it will be inferred from the file extension.")
			.possible_values(Syntax::VALUES)
			.ignore_case(true),
		Arg::new("tags")
			.help("Comma separated list of tags.")
			.long("tags")
			.multiple_values(true)
			.required_unless_present("no-tags")
			.use_value_delimiter(true)
			.validator(validate_tag)
			.require_value_delimiter(true),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let file = m.value_of("path").unwrap();
	let title = m.value_of("title").unwrap();
	let about = m.value_of("about").unwrap();

	let ArticleContents { raw, html, syntax } =
		ArticleContents::read_from_file(file, m.value_of_t("syntax").ok()).await?;

	let url_title = encode_url_title(title);

	let mut tx = db().begin().await?;

	let id = query!(
		"INSERT INTO article(title, url_title, about, raw, html, syntax)
			VALUES($1, $2, $3, $4, $5, $6)
			RETURNING article_id",
		title,
		url_title,
		about,
		raw,
		html,
		syntax as Syntax,
	)
	.fetch_one(&mut tx)
	.await?
	.article_id;

	if let Some(tags) = m.values_of("tags") {
		for tag in tags {
			// Make sure tag exists.
			let affected = query!(
				"INSERT INTO tag(tag_name) VALUES($1) ON CONFLICT(tag_name) DO NOTHING",
				tag
			)
			.execute(&mut tx)
			.await?
			.rows_affected();
			if affected > 0 {
				println!("✓ Created new tag '{}'", tag);
			}
			// Add association entry.
			query!(
				"INSERT INTO article_tag(article_id, tag_name) VALUES($1, $2)",
				id,
				tag
			)
			.execute(&mut tx)
			.await?;
		}
	}

	clear!(articles).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ Published new article '{}' (id = {})", title, id);
	Ok(())
}
