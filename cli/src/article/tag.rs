use super::validate_tag;
use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("tag")
		.about("Display or modify an articles tags.")
		.args(&[
			arg!(article: <ARTICLE> "The articles title or ID."),
			arg!(--clear "Clear all tags from the article."),
			Arg::new("tag")
				.help("Any number of tags to set. Omit to list current tags.")
				.multiple_values(true)
				.validator(validate_tag)
				.conflicts_with("clear"),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let mut tx = db().begin().await?;

	let article = m.value_of("article").unwrap();
	let (id, title) = match article.parse::<i32>() {
		Ok(n) => query!("SELECT title FROM article WHERE article_id = $1", n)
			.fetch_optional(&mut tx)
			.await?
			.map(|x| (n, x.title)),
		Err(_) => query!(
			"SELECT article_id, title FROM article WHERE LOWER(title) = $1",
			article.to_lowercase()
		)
		.fetch_optional(&mut tx)
		.await?
		.map(|x| (x.article_id, x.title)),
	}
	.ok_or_else(|| anyhow!("no article found with the title or ID {}", article))?;

	if m.is_present("clear") {
		let removed = query!(
			"DELETE FROM article_tag WHERE article_id = $1 RETURNING tag_name",
			id
		)
		.fetch_all(&mut tx)
		.await?;
		if removed.is_empty() {
			println!("{} has no tags", &title);
		} else {
			for t in removed {
				println!("✓ untagged: {}", t.tag_name);
			}
		}

		clear!(articles).execute(&mut tx).await?;

		tx.commit().await?;
		return Ok(());
	}

	match m.values_of("tag") {
		None => {
			let tags = query!(
				"SELECT tag_name FROM article_tag WHERE article_id = $1 ORDER BY tag_name ASC",
				id
			)
			.fetch_all(&mut tx)
			.await?;
			for t in tags {
				println!("{}", t.tag_name);
			}
		}
		Some(tags) => {
			query!("DELETE FROM article_tag WHERE article_id = $1", id)
				.execute(&mut tx)
				.await?;

			for tag in tags {
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

				query!(
					"INSERT INTO article_tag(article_id, tag_name) VALUES($1, $2)",
					id,
					tag
				)
				.execute(&mut tx)
				.await?;
			}

			clear!(articles).execute(&mut tx).await?;

			tx.commit().await?;
			println!("Success tagging article '{}'", title);
		}
	}

	Ok(())
}
