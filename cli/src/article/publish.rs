use std::path::PathBuf;

use super::ArticleContents;
use crate::prelude::*;

pub struct Publish {
	db_url: String,
	title: String,
	file: PathBuf,
}

impl Publish {
	pub fn app() -> App<'static> {
		App::new("publish").about("Publish a new article.").args(&[
			arg!(-t --title <TITLE> "The articles title."),
			// arg!(-f --force "Overwrite any existing article with the same title."),
			arg!(file: <FILE> "The article."),
		])
	}
}

impl Publish {
	pub fn from_matches(m: &ArgMatches) -> Self {
		let db_url = m.value_of("database").unwrap().to_string();
		let title = m.value_of("title").unwrap().to_string();
		let file = m.value_of("file").map(PathBuf::from).unwrap();
		Self {
			db_url,
			title,
			file,
		}
	}

	pub fn run(&self) -> Result<()> {
		let ArticleContents {
			markdown,
			html,
			hash,
		} = ArticleContents::read_from_file(&self.file)?;
		let url_title = encode_url_title(&self.title);

		block!(async move {
			let db = init_db(&self.db_url).await?;
			query!(
				"INSERT INTO article(title, url_title, markdown, html, markdown_hash)
			VALUES($1, $2, $3, $4, $5)",
				&self.title,
				url_title,
				markdown,
				html,
				hash,
			)
			.execute(db)
			.await?;
			Ok::<_, anyhow::Error>(())
		})?;

		println!("Success. Published new article titled {}", &self.title);
		Ok(())
	}
}
