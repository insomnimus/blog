use std::borrow::Cow;

use tokio::fs;

use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("fetch")
		.about("Fetch an article from the database.")
		.group(
			ArgGroup::new("handle")
				.required(true)
				.args(&["article", "last"]),
		)
		.args(&[
			arg!(-o --out [FILE] "Save the article to FILE."),
			arg!(article: [ARTICLE] "The ID or the title of the article."),
			arg!(-l --last "Fetch the last article published."),
			arg!(--html "Fetch the rendered HTML of the article instead."),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let id = find_id(m).await?;
	let is_html = m.is_present("html");
	let article = query!(
		r#"SELECT
	title,
	(CASE WHEN NOT $1 THEN raw END) AS raw,
	(CASE WHEN $1 THEN html END) AS html,
	syntax AS "syntax: Syntax"
	FROM article
	WHERE article_id = $2
	"#,
		is_html,
		id,
	)
	.fetch_one(db())
	.await?;

	let out = m.value_of("out").map(Cow::Borrowed).unwrap_or_else(|| {
		if is_html {
			format!("{}.html", format_filename(&article.title)).into()
		} else {
			format!(
				"{}{}",
				format_filename(&article.title),
				article.syntax.ext()
			)
			.into()
		}
	});
	println!("✓ fetched article '{}'", &article.title);

	let data = article.raw.as_ref().or(article.html.as_ref()).unwrap();
	println!("saving article to {out}");
	fs::write(&*out, &data).await?;
	println!("✓ saved article to {out}");
	Ok(())
}

async fn find_id(m: &ArgMatches) -> Result<i32> {
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
			let title = m.value_of("article").unwrap();
			query!(
				"SELECT article_id FROM article WHERE LOWER(title) = $1 OR LOWER(url_title) = $1",
				title.to_lowercase()
			)
			.fetch_optional(db())
			.await?
			.ok_or_else(|| anyhow!("no article found matching the title '{title}'"))?
			.article_id
		}
	};

	Ok(id)
}
