use super::ArticleContents;
use crate::prelude::*;

fn validate_tag(s: &str) -> StdResult<(), String> {
	if s.starts_with(|c: char| c == '-' || c.is_numeric())
		|| s.contains(|c: char| c.is_uppercase() || (c != '-' && !c.is_alphanumeric()))
	{
		Err(String::from("tags can only consist of lowercase letters, numbers and '-' and must start with a lowercase letter"))
	} else {
		Ok(())
	}
}

fn validate_about(s: &str) -> StdResult<(), String> {
	if s.contains(|c: char| c == '\t' || c == '\n' || c == '\r') {
		return Err("the description cannot contain tabs or newlines".into());
	}
	let len = s.chars().count();

	match len {
		0..=14 => Err("the description is too short; at least 15 characters are required".into()),
		15..=120 => Ok(()),
		_ => Err("the description is too long; the value cannot exceed 120 characters".into()),
	}
}

pub fn app() -> App<'static> {
	App::new("publish").about("Publish a new article.").args(&[
		arg!(-f --file <FILE> "The article."),
		arg!(title: <TITLE> "The articles title."),
		arg!(-a --about <DESCRIPTION> "The article description.")
			.validator(validate_about)
			.visible_alias("description"),
		arg!(--"no-tags" "Permit omitting any tag.").conflicts_with("tags"),
		Arg::new("tags")
			.help("Comma separated list of tags.")
			.long("tags")
			.multiple_values(true)
			.required_unless_present("no-tags")
			.use_delimiter(true)
			.validator(validate_tag)
			.require_delimiter(true),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let file = m.value_of("file").unwrap();
	let title = m.value_of("title").unwrap();
	let about = m.value_of("about").unwrap();

	let ArticleContents {
		markdown,
		html,
		hash,
	} = ArticleContents::read_from_file(file).await?;

	let url_title = encode_url_title(title);

	let mut tx = db().begin().await?;

	let id = query!(
		"INSERT INTO article(title, url_title, about, markdown, html, markdown_hash)
			VALUES($1, $2, $3, $4, $5, $6)
			RETURNING article_id",
		title,
		url_title,
		about,
		markdown,
		html,
		hash,
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

	clear_home!().execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ Published new article '{}' (id = {})", title, id);
	Ok(())
}
