mod create;
mod delete;
mod list;

use crate::{
	prelude::*,
	sftp::SendFile,
};

pub fn app() -> App {
	App::new("music")
		.about("Manage music.")
		.subcommand_required(true)
		.arg_required_else_help(true)
			.args(&[
			arg!(--"ssh-config" [PATH] "The Optional ssh_config file, used in commands involving sftp.")
			.global(true),
					arg!(-R --sftp [URL] "The sftp servers connection url in the form `sftp://[user@]domain[:port]/path/to/store`.")
			.env("BLOG_SFTP_URL")
			.global(true),
			arg!(--"sftp-command" [COMMAND] "The sftp command. By default it is `sftp -b -`")
			.validator(validate::<crate::cmd::Cmd>("the sftp command is not valid")),
		])
		.subcommands([create::app(), delete::app(), list::app()])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let db = Config::database(m).await?;
	let c = Config::get_or_init(m.value_of("config")).await?;
	if let Some(cmd) = &c.hooks.pre_db {
		task::block_in_place(|| cmd.to_std().status())?;
	}
	init_db(db).await?;

	match m.subcommand().unwrap() {
		("create", m) => create::run(m).await,
		("delete", m) => delete::run(m).await,
		("list", m) => list::run(m).await,
		_ => unreachable!(),
	}
}

#[derive(Debug, Serialize)]
struct Music {
	pub id: i32,
	pub title: Option<String>,
	pub comment: Option<String>,
	pub date: String,
	pub media: String,
}

impl Formattable for Music {
	fn human(&self) -> String {
		let comment = self
			.comment
			.as_ref()
			.map_or_else(String::new, |s| match s.len() {
				0..=30 => format!(" - {}...", &s[..(s.len().min(27))]),
				_ => format!(" - {s}"),
			});
		let id = self.id;
		let title = self.title.as_deref().unwrap_or("untitled");
		let date = &self.date;
		let media = &self.media;

		format!("#{id}> {title} on {date} ({media}){comment}")
	}
}

impl Tsv for Music {
	fn tsv(&self) -> String {
		format!(
			"{id}\t{title}\t{date}\t{media}\t{comment}",
			id = self.id,
			date = self.date.tsv(),
			title = self.title.tsv(),
			comment = self.comment.tsv(),
			media = self.media.tsv(),
		)
	}
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
