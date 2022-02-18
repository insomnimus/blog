use crate::prelude::*;

pub fn app() -> App {
	App::new("fetch").about("Download the about page.").args(&[
		arg!(--rendered "Download the rendered page (html) instead."),
		arg!(-o --out [PATH] "Save the contents to a file."),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let data = query!("SELECT html, raw FROM about")
		.fetch_optional(db())
		.await?
		.ok_or_else(|| {
			anyhow!("the database does not contain an about page; create one with the `edit` subcommand")
		})?;

	let contents = if m.is_present("rendered") {
		&data.html
	} else {
		&data.raw
	};

	match m.value_of("out") {
		None => print!("{contents}"),
		Some(p) => {
			tokio::fs::write(p, contents).await?;
			println!("✓ saved the contents to {p}");
		}
	}

	Ok(())
}
