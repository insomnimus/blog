use clap::crate_version;

use crate::{
	article::ArticleCmd,
	prelude::*,
};

pub enum Cmd {
	Article(ArticleCmd),
}

impl Cmd {
	pub fn from_args() -> Self {
		let m = App::new("blog")
			.about("Blog management cli.")
			.version(crate_version!())
			.setting(AppSettings::InferSubcommands)
			.setting(AppSettings::SubcommandRequiredElseHelp)
			.global_setting(AppSettings::PropagateVersion)
			.subcommands([ArticleCmd::app()])
			.get_matches();

		match m.subcommand().unwrap() {
			("article", m) => Self::Article(ArticleCmd::from_matches(m)),
			_ => unreachable!(),
		}
	}

	pub fn run(self) -> Result<()> {
		match self {
			Self::Article(x) => x.run(),
		}
	}
}
