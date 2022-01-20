use super::{
	validate_about,
	validate_tag,
	validate_title,
};
use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("edit")
		.about("Update an existing article.")
		.group(
			ArgGroup::new("md")
				.multiple(true)
				.required(true)
				.args(&["path", "title", "tags", "about"]),
		)
		.args(&[
			arg!(article: <ARTICLE> "The ID or title of the article.").required(true),
			arg!(-p --path [FILE] "The path to the file containing the new article contents."),
			arg!(-t --title [NEW_TITLE] "The new article title.").validator(validate_title),
			Arg::new("tags")
				.long("tags")
				.help("Comma separated list of tags.")
				.multiple_values(true)
				.use_delimiter(true)
				.require_delimiter(true)
				.validator(validate_tag),
			arg!(-a --about [DESCRIPTION] "Change the article description.")
				.validator(validate_about),
		])
}
