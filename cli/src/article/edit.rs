use super::{
	validate_about,
	validate_title,
	ArticleContents,
};
use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("edit")
		.about("Update an existing article.")
		.group(
			ArgGroup::new("md")
				.multiple(true)
				.required(true)
				.args(&["path", "title", "about"]),
		)
		.args(&[
			arg!(article: <ARTICLE> "The ID or title of the article to edit.").required(true),
			arg!(-p --path [FILE] "The path to the file containing the new article contents."),
			arg!(-t --title [NEW_TITLE] "The new article title.").validator(validate_title),
			arg!(-a --about [DESCRIPTION] "Change the article description.")
				.validator(validate_about),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let (html, markdown, hash) = match m.value_of("path") {
		None => (None, None, None),
		Some(p) => ArticleContents::read_from_file(p).await.map(
			|ArticleContents {
			     markdown,
			     html,
			     hash,
			 }| (Some(html), Some(markdown), Some(hash)),
		)?,
	};

	let mut tx = db().begin().await?;
	let id = match m.value_of_t::<i32>("article") {
		Ok(n) => n,
		Err(_) => {
			query!(
				"SELECT article_id FROM article WHERE LOWER(title) = $1 LIMIT 1",
				m.value_of("article").unwrap().to_lowercase()
			)
			.fetch_optional(&mut tx)
			.await?
			.ok_or_else(|| {
				anyhow!(
					"could not find an article titled '{}'",
					m.value_of("article").unwrap()
				)
			})?
			.article_id
		}
	};

	let title = m.value_of("title").unwrap_or_default();
	let url_title = encode_url_title(title);
	let about = m.value_of("about");

	query!(
		"UPDATE article
	SET
	title = COALESCE(NULLIF($1, ''), title),
	url_title = COALESCE(NULLIF($2, ''), url_title),
	about = COALESCE($3, about),
	html = COALESCE($4, html),
	markdown_hash = COALESCE($5, markdown_hash),
	markdown = COALESCE($6, markdown),
	date_updated = NOW() AT TIME ZONE 'UTC'
	WHERE article_id = $7",
		&title,
		&url_title,
		about,
		html.as_ref(),
		hash.as_ref(),
		markdown.as_ref(),
		id,
	)
	.execute(&mut tx)
	.await?
	.rows_affected();

	tx.commit().await?;

	macro_rules! print_if {
		($arg:expr, $msg:expr) => {
			if m.is_present($arg) {
				println!("✓ updated {}", $msg);
			}
		};
	}

	print_if!("about", "about article");
	print_if!("title", "the article title");
	print_if!("path", "article contents");

	Ok(())
}
