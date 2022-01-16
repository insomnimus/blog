use super::ArticleContents;
use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("publish").about("Publish a new article.").args(&[
		arg!(-f --file <FILE> "The article."),
		arg!(title: <TITLE> "The articles title."),
		// arg!(-f --force "Overwrite any existing article with the same title."),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let file = m.value_of("file").unwrap();
	let title = m.value_of("title").unwrap();

	let ArticleContents {
		markdown,
		html,
		hash,
	} = ArticleContents::read_from_file(file).await?;

	let url_title = encode_url_title(title);

	let mut tx = db().begin().await?;

	query!(
		"INSERT INTO article(title, url_title, markdown, html, markdown_hash)
			VALUES($1, $2, $3, $4, $5)",
		title,
		url_title,
		markdown,
		html,
		hash,
	)
	.execute(&mut tx)
	.await?;

	clear_home!().execute(&mut tx).await?;
	tx.commit().await?;

	println!("Success. Published new article titled {}", title);
	Ok(())
}
