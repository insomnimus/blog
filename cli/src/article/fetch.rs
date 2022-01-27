use tokio::fs;

use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("fetch")
		.about("Fetch an article from the database.")
		.args(&[
			arg!(-o --out <FILE> "Save the article as a markdown document to FILE."),
			arg!(article: <ARTICLE> "The ID or the title of the article."),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let article = m.value_of("article").unwrap();
	let out = m.value_of("out").unwrap();

	let res = match article.parse::<u32>() {
		Ok(id) => query!(
			"SELECT raw, title FROM article WHERE article_id = $1",
			id as i32
		)
		.fetch_optional(db())
		.await?
		.map(|mut x| (x.title.take(), x.raw.take())),
		Err(_) => query!(
			"SELECT raw, title
					FROM article
					WHERE
					(LOWER(title) = $1)
					OR
					(LOWER(url_title) = $1)
					LIMIT 1",
			article.to_lowercase(),
		)
		.fetch_optional(db())
		.await?
		.map(|mut x| (x.title.take(), x.raw.take())),
	};

	match res {
		None => Err(anyhow!("No article found by the ID or title {}", article,)),
		Some((title, raw)) => {
			println!("Saving article '{}'", &title);
			fs::write(out, &raw).await?;
			Ok(())
		}
	}
}
