use super::Post;
use crate::prelude::*;

pub fn app() -> App {
	App::new("list").about("List notes.").args(&[
		arg!(--oldest "Show oldest posts first."),
		arg!(--rendered "Include the rendered HTML in the output."),
		arg!(n: -n [N] "Show first N notes, 0 for all.")
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
		n.note_id AS id,
		n.date_posted AS date,
		n.raw,
		(CASE WHEN $1 THEN n.content END) AS content,
		ARRAY_AGG(m.file_path) AS "attachments: Vec<Option<String>>"
		FROM note n
		LEFT JOIN note_media m
		ON n.note_id = m.note_id
		GROUP BY n.note_id
		ORDER BY
		CASE WHEN $2 THEN n.note_id END ASC,
		CASE WHEN NOT $2 THEN n.note_id END DESC
		LIMIT $3"#,
		rendered,
		oldest,
		n,
	)
	.fetch(db());

	while let Some(mut x) = results.next().await.transpose()? {
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
