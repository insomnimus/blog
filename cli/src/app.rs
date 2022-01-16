use clap::crate_version;

use crate::{
	article,
	prelude::*,
};

pub fn app() -> App<'static> {
	App::new("blog")
		.about("Blog management cli.")
		.version(crate_version!())
		.setting(AppSettings::SubcommandRequiredElseHelp)
		.global_setting(AppSettings::InferSubcommands)
		.global_setting(AppSettings::PropagateVersion)
		.subcommands([article::app()])
}

pub async fn run() -> Result<()> {
	let m = app().get_matches();
	match m.subcommand().unwrap() {
		("article", m) => article::run(m).await,
		_ => unreachable!(),
	}
}
