mod delete;
mod edit;
mod fetch;
mod list;
mod publish;

use std::path::Path;

use pulldown_cmark::{
	html::push_html,
	Options,
	Parser,
};
use serde::Serialize;
use sha2::{
	Digest,
	Sha256,
};
use tokio::{
	fs,
	io,
};

use crate::prelude::*;

pub fn app() -> App<'static> {
	App::new("article")
		.about("Manage articles.")
		.setting(AppSettings::SubcommandRequiredElseHelp)
		.arg(
			arg!(-D --database <URL> "Database URL.")
				.env("BLOG_DB_URL")
				.hide_env_values(true),
		)
		.subcommands([
			delete::app(),
			edit::app(),
			fetch::app(),
			list::app(),
			publish::app(),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	init_db(m.value_of("database").unwrap()).await?;

	match m.subcommand().unwrap() {
		("delete", m) => delete::run(m).await,
		("edit", m) => edit::run(m).await,
		("fetch", m) => fetch::run(m).await,
		("list", m) => list::run(m).await,
		("publish", m) => publish::run(m).await,
		_ => unreachable!(),
	}
}

#[derive(Default)]
struct ArticleContents {
	markdown: String,
	html: String,
	hash: Vec<u8>,
}

impl ArticleContents {
	async fn read_from_file<P: AsRef<Path>>(p: P) -> io::Result<Self> {
		let data = fs::read_to_string(p.as_ref()).await?;
		let mut html = String::new();
		let mut hasher = Sha256::new();
		hasher.update(data.trim().as_bytes());
		let opts = Options::all();
		let parser = Parser::new_ext(&data, opts);
		push_html(&mut html, parser);

		Ok(Self {
			markdown: data,
			hash: hasher.finalize().to_vec(),
			html,
		})
	}
}

#[derive(Serialize)]
pub struct ArticleInfo {
	id: i32,
	title: String,
	about: String,
	url_title: String,
	published: String,
	updated: Option<String>,
	tags: Vec<String>,
}

impl Formattable for ArticleInfo {
	fn human(&self) -> String {
		format!(
			"#{id}> {title} - {about} ({date})",
			id = self.id,
			title = &self.title,
			date = &self.published,
			about = &self.about
		)
	}
}

impl Tsv for ArticleInfo {
	fn tsv(&self) -> String {
		format!(
			"{id}\t{title}\t{published}\t{updated}\t{about}\t{tags}\t{url_title}",
			about = self.about.tsv(),
			id = self.id,
			title = self.title.tsv(),
			published = &self.published,
			updated = self.updated.as_deref().tsv(),
			tags = self.tags.tsv(),
			url_title = self.url_title.tsv(),
		)
	}
}

fn validate_tag(s: &str) -> StdResult<(), String> {
	if s.starts_with(|c: char| c == '-' || c.is_numeric())
		|| s.contains(|c: char| c.is_uppercase() || (c != '-' && !c.is_alphanumeric()))
	{
		Err(String::from("tags can only consist of lowercase letters, numbers and '-' and must start with a lowercase letter"))
	} else {
		Ok(())
	}
}

fn validate_about(s: &str) -> StdResult<(), String> {
	if s.contains(|c: char| c == '\t' || c == '\n' || c == '\r') {
		return Err("the description cannot contain tabs or newlines".into());
	}
	let len = s.chars().count();

	match len {
		0..=14 => Err("the description is too short; at least 15 characters are required".into()),
		15..=120 => Ok(()),
		_ => Err("the description is too long; the value cannot exceed 120 characters".into()),
	}
}

fn validate_title(s: &str) -> StdResult<(), String> {
	if s.contains(|c: char| "\r\n\t".contains(c)) {
		Err(String::from("the title cannot contain tabs or newlines"))
	} else if s.trim().len() < 3 {
		Err(String::from("the title must be at least 3 characters long"))
	} else {
		Ok(())
	}
}
