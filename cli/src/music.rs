mod create;

use crate::{
	prelude::*,
	sftp::SendFile,
};

pub fn app() -> App<'static> {
	App::new("music")
		.about("Manage music.")
		.setting(AppSettings::SubcommandRequiredElseHelp)
		.arg(
			arg!(-D --database [URL] "The database URL with write permissions.")
				.env("BLOGCLI_DB_URL")
				.hide_env_values(true),
		)
		.subcommands([create::app()])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let db = Config::database(m).await?;
	init_db(db).await?;

	match m.subcommand().unwrap() {
		("create", m) => create::run(m).await,
		_ => unreachable!(),
	}
}

struct Music {
	pub id: i32,
	pub title: Option<String>,
	pub comment: Option<String>,
	pub date: String,
	pub media: String,
}

fn validate_music(s: &str) -> StdResult<(), String> {
	match s.parse::<SendFile>() {
		Err(e) => Err(e.to_string()),
		Ok(f) => {
			use std::path::Path;

			let exts = ["mp3", "webm", "mp4", "wav", "opus"];
			let local_ext = Path::new(f.local());
			let local_ext = local_ext.extension().unwrap_or_default();
			let remote_ext = Path::new(f.remote());
			let remote_ext = remote_ext.extension().unwrap_or_default();

			if !exts.iter().any(|ext| local_ext.eq_ignore_ascii_case(ext)) {
				Err(format!(
					"the file extension is not supported; it must be one of {exts:?}"
				))
			} else if local_ext != remote_ext {
				Err(String::from(
					"remote file extension must be the same with the original",
				))
			} else {
				Ok(())
			}
		}
	}
}
