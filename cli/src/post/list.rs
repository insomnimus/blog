use super::Post;
use crate::prelude::*;

pub fn app() -> App {
	App::new("list").about("List posts.").args(&[
		arg!(--oldest "Show oldest posts first."),
		arg!(--rendered "Include the rendered HTML in the output."),
		arg!(n: -n [N] "Show first N posts, 0 for all.")
			.default_value("10")
			.validator(validate::<u32>("the value must be a positive integer or 0")),
		arg!(-f --format [FORMAT] "The output format.")
			.possible_values(Format::VALUES)
			.default_value("human")
			.ignore_case(true),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let format = m.value_of_t_or_exit::<Format>("format");
	let oldest = m.is_present("oldest");
	let rendered = m.is_present("rendered");
	let n = m.value_of_t_or_exit::<u32>("n") as i64;
	let n = if n == 0 { 30000_i64 } else { n as i64 };

	let mut results = query!(
		r#"SELECT
		p.post_id AS id,
		p.date_posted AS date,
		p.raw,
		(CASE WHEN $1 THEN p.content END) AS content,
		ARRAY_AGG(m.file_path) AS "attachments: Vec<Option<String>>"
		FROM post p
		LEFT JOIN post_media m
		ON p.post_id = m.post_id
		GROUP BY p.post_id
		ORDER BY
		CASE WHEN $2 THEN p.post_id END ASC,
		CASE WHEN NOT $2 THEN p.post_id END DESC
		LIMIT $3"#,
		rendered,
		oldest,
		n,
	)
	.fetch(db());

	while let Some(res) = results.next().await {
		let mut x = res?;
		let p = Post {
			id: x.id,
			date: x.date.to_local(),
			attachments: x
				.attachments
				.take()
				.into_iter()
				.flatten()
				.flatten()
				.collect(),
			rendered: x.content.take(),
			raw: x.raw.take(),
		};

		p.print(format)?;
	}

	Ok(())
}
