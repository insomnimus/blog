use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("fetch").about("Download the about page.").args(&[
		arg!(--rendered "Download the rendered page (html) instead."),
		arg!(-o --out [PATH] "Save the contents to a file."),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let data =
		query!(
			r#"SELECT
	(CASE WHEN $1 THEN html END,
	CASE WHEN 'TRUE' THEN raw END) AS "contents!: String"
	FROM about"#,
			m.is_present("rendered"),
		)
		.fetch_optional(db())
		.await?
		.ok_or_else(|| {
			anyhow!("the database does not contain an about page; create one with the `edit` subcommand")
		})?
		.contents;

	match m.value_of("out") {
		None => println!("{data}"),
		Some(p) => {
			tokio::fs::write(p, &data).await?;
			println!("✓ saved the contents to {p}");
		}
	}

	Ok(())
}
