mod create;
mod delete;
mod edit;
mod list;

use crate::prelude::*;

pub fn app() -> App {
	App::new("post")
		.about("Manage short posts.")
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
		.subcommands([create::app(), delete::app(), edit::app(), list::app()])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let db = Config::database(m).await?;
	run_hook!(pre_db, m).await?;

	init_db(db).await?;

	match m.subcommand().unwrap() {
		("create", m) => create::run(m).await,
		("delete", m) => delete::run(m).await,
		("edit", m) => edit::run(m).await,
		("list", m) => list::run(m).await,
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

#[derive(Serialize)]
struct Post {
	id: i32,
	date: String,
	raw: String,
	rendered: Option<String>,
	attachments: Vec<String>,
}

impl Tsv for Post {
	fn tsv(&self) -> String {
		format!(
			"{id}\t{raw}\t{date}\t{attachments}\t{rendered}",
			id = self.id,
			raw = self.raw.tsv(),
			date = self.date.tsv(),
			rendered = self.rendered.as_deref().tsv(),
			attachments = self.attachments.tsv(),
		)
	}
}

impl Formattable for Post {
	fn human(&self) -> String {
		let attachments = match self.attachments.len() {
			0 => String::new(),
			1 => String::from(" (has 1 attachment)"),
			n => format!(" (has {n} attachments)"),
		};
		format!(
			"#{id} on {date}{attachments}:
	{raw}",
			id = self.id,
			raw = &self.raw,
			date = &self.date,
		)
	}
}

fn post_dir(id: i32) -> String {
	format!("post_{id}")
}
