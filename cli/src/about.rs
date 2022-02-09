mod edit;
mod fetch;

use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("about")
		.about("Update or fetch the about page.")
		.setting(AppSettings::SubcommandRequiredElseHelp)
		.subcommands([edit::app(), fetch::app()])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let db = Config::database(m).await?;
	init_db(db).await?;

	match m.subcommand().unwrap() {
		("edit", m) => edit::run(m).await,
		("fetch", m) => fetch::run(m).await,
		_ => unreachable!(),
	}
}
