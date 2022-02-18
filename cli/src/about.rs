mod edit;
mod fetch;

use crate::prelude::*;

pub fn app() -> App {
	App::new("about")
		.about("Update or fetch the about page.")
		.subcommand_required(true)
		.arg_required_else_help(true)
		.subcommands([edit::app(), fetch::app()])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let db = Config::database(m).await?;
	run_hook!(pre_db, m).await?;
	init_db(db).await?;

	match m.subcommand().unwrap() {
		("edit", m) => edit::run(m).await,
		("fetch", m) => fetch::run(m).await,
		_ => unreachable!(),
	}
}
