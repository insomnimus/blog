pub fn app() -> App<'static> {
	App::new("info")
	.about("Show information about published articles.")
	.group(ArgGroup::new("handle")
	.required(true)
	.args(&["article", "last"])
	)
	.args(&[
	arg!(-f --format [FORMAT] "The message format.")
	.default_value("human")
	.possible_values(Format::VALUES)
	.ignore_case(true),
	arg!(article: [ARTICLE] "The article id or title.")
	arg!(-l --last "Show info about the last article published."),
	])
}
