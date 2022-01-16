use super::ArticleInfo;
use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("list")
		.about("List metadata about published articles.")
		.args(&[
			arg!(--oldest "Show oldest articles first."),
			arg!(n: -n [N] "Show N articles, 0 for all.")
				.default_value("5")
				.validator(validate::<usize>(
					"The value must be a positive integer or 0.",
				)),
			arg!(-f --format [FORMAT] "The output format.")
				.possible_values(Format::VALUES)
				.default_value("human")
				.ignore_case(true),
			arg!(filter: [FILTER] "A glob pattern to match against titles."),
			Arg::new("tags")
				.help("Comma separated list of tags to search.")
				.long("tags")
				.multiple_values(true)
				.use_delimiter(true),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let format = m.value_of_t_or_exit::<Format>("format");
	// let oldest = if m.is_present("oldest") { 1 } else { -1 };
	let oldest = m.is_present("oldest");
	let tags: Vec<_> = m
		.values_of("tags")
		.map(|i| i.map(String::from).collect())
		.unwrap_or_default();
	let filter = m
		.value_of("filter")
		.map(str::to_lowercase)
		.unwrap_or_default();
	let n = m.value_of_t_or_exit::<i32>("n");
	let n = if n == 0 { 30000_i64 } else { n as i64 };

	let vals = query!(
		"SELECT
	a.article_id, a.title, a.url_title, a.date_updated, a.date_published,
	ARRAY_AGG(t.tag_name) tags_array
	FROM article a
	LEFT JOIN article_tag t
	ON a.article_id = t.article_id
	WHERE $1 = '' OR LOWER(a.title) SIMILAR TO $1
	GROUP BY a.title, a.article_id, a.url_title
	HAVING ARRAY_AGG(t.tag_name) @> $2
	ORDER BY
	CASE WHEN $3 = TRUE THEN a.date_published END ASC,
	CASE WHEN $3 = FALSE THEN a.date_published END DESC
	LIMIT $4",
		filter,
		tags.as_slice(),
		oldest,
		n,
	)
	.fetch_all(db())
	.await?;

	for mut x in vals {
		let info = ArticleInfo {
			id: x.article_id,
			title: mem::take(&mut x.title),
			url_title: mem::take(&mut x.url_title),
			tags: x.tags_array.take().unwrap_or_default(),
			published: format_date(x.date_published),
			updated: x.date_updated.map(format_date),
		};

		format.print(&info)?;
	}
	Ok(())
}
