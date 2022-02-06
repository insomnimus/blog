use super::Music;
use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("list")
		.about("List published music posts.")
		.args(&[
			arg!(--oldest "Show oldest music first."),
			arg!(-f --format [FORMAT] "The output format.")
				.default_value("human")
				.possible_values(Format::VALUES)
				.ignore_case(true),
			arg!(n: -n [LIMIT] "Output limit, 0 means no limit.").validator(validate::<usize>(
				"the value must be a positive integer or 0",
			)),
			arg!(-c --comments "Include comments in the output."),
			arg!(filter: [FILTER] "A postgresql style glob pattern to match against titles."),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let format = m.value_of_t_or_exit::<Format>("format");
	let oldest = m.is_present("oldest");
	let comments = m.is_present("comments");
	let filter = m
		.value_of("filter")
		.map(str::to_lowercase)
		.unwrap_or_default();
	let n = m
		.value_of_t::<i64>("n")
		.unwrap_or(if filter.is_empty() { 5 } else { 0 });

	let n = if n == 0 { 900000 } else { n };

	let mut results = query!(
		r#"SELECT
	music_id AS id,
	date_uploaded AS date,
	title,
	file_path AS media,
	(CASE WHEN $1 THEN comment END) AS comment
	FROM music
	WHERE $2 = '' OR LOWER(title) SIMILAR TO $2
	ORDER BY
	CASE WHEN $3 THEN date_uploaded END ASC,
	 CASE WHEN 'TRUE' THEN date_uploaded END DESC
	LIMIT $4"#,
		comments,
		&filter,
		oldest,
		n,
	)
	.fetch(db());

	while let Some(res) = results.next().await {
		let mut x = res?;
		let music = Music {
			id: x.id,
			date: x.date.to_local(),
			comment: x.comment.take(),
			title: x.title.take(),
			media: x.media.take(),
		};

		music.print(format)?;
	}

	Ok(())
}
