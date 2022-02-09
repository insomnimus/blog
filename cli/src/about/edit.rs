use crate::{
	article::ArticleContents,
	prelude::*,
};

pub fn app() -> App<'static> {
	App::new("edit")
	.about("Update the about page.")
	.args(&[
	arg!(path: [FILE] "Path to a file containing the new about page body. Omit this to edit the current page with your editor."),
	arg!(-s --syntax [SYNTAX] "The markup syntax of the document.")
	.possible_values(Syntax::VALUES)
	.ignore_case(true),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let ArticleContents { raw, html, syntax } = match m.value_of("path") {
		Some(p) => ArticleContents::read_from_file(p, m.value_of_t("syntax").ok()).await?,
		None => {
			let res = query!(r#"SELECT raw, syntax AS "syntax: Syntax" FROM about"#)
				.fetch_optional(db())
				.await?
				.ok_or_else(|| {
					anyhow!("there is no about page in the database; pelase specify --path to create it")
				})?;
			let syntax = m.value_of_t("syntax").unwrap_or(res.syntax);
			let raw = edit_buf("about_page_", syntax.ext(), &res.raw).await?;
			match raw {
				Some(raw) => ArticleContents::new(raw, syntax),
				None => return Ok(()),
			}
		}
	};

	query!(
		"INSERT INTO about(raw, html, syntax)
			VALUES($1, $2, $3)
			ON CONFLICT(_instance) DO UPDATE SET
			raw = $1,
			html = $2,
			syntax = $3,
			last_updated = NOW() AT TIME ZONE 'UTC'",
		raw,
		html,
		syntax as Syntax,
	)
	.execute(db())
	.await?;

	println!("✓ updated the about page");
	Ok(())
}
