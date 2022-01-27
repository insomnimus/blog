use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("delete")
		.about("Delete an article.")
		.args(&[
			arg!(-i --id [ID] "The article ID.")
				.validator(validate::<u32>("the value must be a positive integer or 0")),
			arg!(-t --title [TITLE] "The full title of the article."),
			arg!(-f --force "Do not ask for confirmation."),
		])
		.group(
			ArgGroup::new("article")
				.args(&["id", "title"])
				.required(true),
		)
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	struct Article {
		title: String,
		article_id: i32,
	}
	let mut tx = db().begin().await?;
	let info = match m.value_of("title") {
		Some(title) => {
			query_as!(
				Article,
				"SELECT article_id, title FROM article WHERE LOWER(title) = $1",
				title.to_lowercase()
			)
			.fetch_optional(&mut tx)
			.await?
		}
		None => {
			query_as!(
				Article,
				"SELECT article_id, title FROM article WHERE article_id = $1",
				m.value_of_t::<i32>("id")?
			)
			.fetch_optional(&mut tx)
			.await?
		}
	}
	.ok_or_else(|| anyhow!("no article found"))?;

	if !(m.is_present("force")
		|| confirm!(
			"Are you sure you want to delete '{}' (id = {})?",
			&info.title,
			info.article_id
		)?) {
		return Ok(());
	}

	query!("DELETE FROM article WHERE article_id = $1", info.article_id)
		.execute(&mut tx)
		.await?;
	clear!(articles).execute(&mut tx).await?;
	tx.commit().await?;

	println!("✓ Deleted article '{}'", &info.title);
	Ok(())
}
