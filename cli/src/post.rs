mod create;

use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("post")
		.about("Manage short posts.")
		.setting(AppSettings::SubcommandRequiredElseHelp)
		.arg(
			arg!(-D --database <URL> "Database URL.")
				.env("BLOG_DB_URL")
				.hide_env_values(true),
		)
		.subcommands([create::app()])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	init_db(m.value_of("database").unwrap()).await?;

	match m.subcommand().unwrap() {
		("create", m) => create::run(m).await,
		_ => unreachable!(),
	}
}

fn validate_post(s: &str) -> StdResult<(), String> {
	match s.trim().chars().count() {
		0..=4 => Err("the post is too short; it must be at least 5 characters".into()),
		5..=400 => Ok(()),
		_ => Err("post is too long; it can't exceed 400 characters".into()),
	}
}
