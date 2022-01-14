use std::fs;

use crate::prelude::*;

pub struct Fetch {
	handle: String,
	out: String,
}

impl Fetch {
	pub fn app() -> App<'static> {
		App::new("fetch")
			.about("Fetch an article from the database.")
			.args(&[
				arg!(-o --out <FILE> "Save the article as a markdown document to FILE."),
				arg!(article: <ARTICLE> "The ID or the title of the article."),
			])
	}

	pub fn from_matches(m: &ArgMatches) -> Self {
		let handle = m.value_of("article").unwrap().to_string();
		let out = m.value_of("out").unwrap().to_string();

		Self { handle, out }
	}
}

impl Fetch {
	pub fn run(self) -> Result<()> {
		let res = match self.handle.parse::<u32>() {
			Ok(id) => {
				block!(async move {
					query!(
						"SELECT markdown, title FROM article WHERE article_id = $1",
						id as i32
					)
					.fetch_optional(db())
					.await
					.map(|x| x.map(|mut x| (mem::take(&mut x.title), mem::take(&mut x.markdown))))
				})
			}
			Err(_) => {
				block!(async {
					query!(
						"SELECT markdown, title
					FROM article
					WHERE
					(LOWER(title) = LOWER($1))
					OR
					(LOWER(url_title) = LOWER($1))
					LIMIT 1",
						&self.handle
					)
					.fetch_optional(db())
					.await
					.map(|x| x.map(|mut x| (mem::take(&mut x.title), mem::take(&mut x.markdown))))
				})
			}
		}?;

		match res {
			None => Err(anyhow!(
				"No article found by the ID or title {}",
				&self.handle
			)),
			Some((title, markdown)) => {
				println!("Saving article '{}'", &title);
				fs::write(&self.out, &markdown)?;
				Ok(())
			}
		}
	}
}
