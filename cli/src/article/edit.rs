use super::{
	validate_about,
	validate_title,
	ArticleContents,
};
use crate::prelude::*;

pub fn app() -> App {
	App::new("edit")
		.about("Update an existing article.")
		.group(
			ArgGroup::new("handle")
				.required(true)
				.args(&["article", "last"]),
		)
		.group(ArgGroup::new("edit")
		.args(&["editor", "path"]))
		.group(ArgGroup::new("md")
		.multiple(true)
		.required(true)
		.args(&["editor", "path", "title", "about"])
		)
		.args(&[
			arg!(article: [ARTICLE] "The ID or title of the article to edit."),
			arg!(--last "Edit the last article published."),
			arg!(-p --path [FILE] "The path to the file containing the new article contents."),
			arg!(-e --editor "Edit the article in your editor."),
			arg!(-s --syntax [SYNTAX] "The syntax for rendering. If omitted and --path is set, it will be inferred from the file extension.")
			.possible_values(Syntax::VALUES)
			.ignore_case(true),
			arg!(-t --title [NEW_TITLE] "The new article title.").validator(validate_title),
			arg!(-a --about [DESCRIPTION] "Change the article description.")
				.validator(validate_about),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	if m.is_present("editor") {
		run_editor(m).await?;
	} else {
		run_no_editor(m).await?;
	}
	Ok(())
}

async fn run_editor(m: &ArgMatches) -> Result<()> {
	let (id, raw, syntax) = match m.value_of_t::<i32>("article") {
		Ok(id) => query!(r#"SELECT raw, syntax AS "syntax: Syntax" FROM article WHERE article_id = $1"#, id)
			.fetch_optional(db())
			.await?
			.map(|mut x| (id, x.raw.take(), x.syntax))
			.ok_or_else(|| anyhow!("no article found with the id {id}"))?,
		Err(_) if m.is_present("last") => {
			query!(r#"SELECT article_id, raw, syntax AS "syntax: Syntax" FROM article ORDER BY article_id DESC LIMIT 1"#)
				.fetch_optional(db())
				.await?
				.map(|mut x| (x.article_id, x.raw.take(), x.syntax))
				.ok_or_else(|| anyhow!("there are no articles in the database"))?
		}
		Err(_) => {
			let name = m.value_of("article").unwrap();

			query!(
				r#"SELECT article_id,
			raw,
			syntax AS "syntax: Syntax"
			FROM article
			WHERE LOWER(title) = $1"#,
				name.to_lowercase(),
			)
			.fetch_optional(db())
			.await?
			.map(|mut x| (x.article_id, x.raw.take(), x.syntax))
			.ok_or_else(|| anyhow!("no article found with the title '{name}'"))?
		}
	};

	let syntax = m.value_of_t::<Syntax>("syntax").unwrap_or(syntax);

	let raw = match edit_buf("article", syntax.ext(), &raw).await? {
		None => return Ok(()),
		Some(x) => x,
	};

	let contents = ArticleContents::new(raw, syntax);
	update_article(m, id, Some(contents)).await
}

async fn run_no_editor(m: &ArgMatches) -> Result<()> {
	let contents = match m.value_of("path") {
		None => None,
		Some(p) => Some(ArticleContents::read_from_file(p, m.value_of_t("syntax").ok()).await?),
	};

	let id = match m.value_of_t::<i32>("article") {
		Ok(id) => {
			query!("SELECT article_id FROM article WHERE article_id = $1", id)
				.fetch_optional(db())
				.await?
				.ok_or_else(|| anyhow!("no article found with the id {id}"))?;

			id
		}
		Err(_) if m.is_present("last") => {
			query!("SELECT article_id FROM article ORDER BY article_id DESC LIMIT 1")
				.fetch_optional(db())
				.await?
				.ok_or_else(|| anyhow!("there are no articles in the database"))?
				.article_id
		}
		Err(_) => {
			let name = m.value_of("article").unwrap();
			query!(
				"SELECT article_id FROM article WHERE LOWER(title) = $1",
				name.to_lowercase()
			)
			.fetch_optional(db())
			.await?
			.ok_or_else(|| anyhow!("no article found matching the title '{name}'"))?
			.article_id
		}
	};

	update_article(m, id, contents).await
}

async fn update_article(m: &ArgMatches, id: i32, contents: Option<ArticleContents>) -> Result<()> {
	let title = m.value_of("title");
	let url_title = title.map(encode_url_title);

	let about = m.value_of("about");

	let mut tx = db().begin().await?;

	let title = query!(
		r#"UPDATE article
	SET
	title = COALESCE($1, title),
	url_title = COALESCE($2, url_title),
	about = COALESCE($3, about),
	html = COALESCE($4, html),
	raw = COALESCE($5, raw),
	syntax = COALESCE($6, syntax),
	date_updated = NOW() AT TIME ZONE 'UTC'
	WHERE article_id = $7
	RETURNING title"#,
		title,
		url_title,
		about,
		contents.as_ref().map(|c| c.html.as_str()),
		contents.as_ref().map(|c| c.raw.as_str()),
		contents.as_ref().map(|c| c.syntax) as Option<Syntax>,
		id,
	)
	.fetch_one(&mut tx)
	.await?
	.title;

	clear!(articles).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ updated article '{title}'");
	Ok(())
}
