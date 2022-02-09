mod create;
mod delete;
mod edit;
mod fetch;
mod list;
mod tag;

use std::path::Path;

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
		.subcommands([
			create::app(),
			delete::app(),
			edit::app(),
			fetch::app(),
			list::app(),
			tag::app(),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let db = Config::database(m).await?;
	init_db(db).await?;

	match m.subcommand().unwrap() {
		("create", m) => create::run(m).await,
		("delete", m) => delete::run(m).await,
		("edit", m) => edit::run(m).await,
		("fetch", m) => fetch::run(m).await,
		("list", m) => list::run(m).await,
		("tag", m) => tag::run(m).await,
		_ => unreachable!(),
	}
}

struct ArticleContents {
	raw: String,
	html: String,
	hash: Vec<u8>,
	syntax: Syntax,
}

impl ArticleContents {
	fn new<S: Into<String>>(raw: S, syntax: Syntax) -> Self {
		let raw = raw.into();
		let mut hasher = Sha256::new();
		hasher.update(raw.trim().as_bytes());
		let html = syntax.render(&raw).into_owned();

		Self {
			raw,
			hash: hasher.finalize().to_vec(),
			html,
			syntax,
		}
	}

	async fn read_from_file<P: AsRef<Path>>(p: P, syntax: Option<Syntax>) -> io::Result<Self> {
		let data = fs::read_to_string(p.as_ref()).await?;
		let syntax = syntax.unwrap_or_else(|| {
			Syntax::from_ext(
				p.as_ref()
					.extension()
					.map_or("txt", |ext| ext.to_str().unwrap_or("txt")),
			)
			.unwrap_or(Syntax::Plain)
		});

		let mut hasher = Sha256::new();
		hasher.update(data.trim().as_bytes());
		let html = syntax.render(&data).into_owned();

		Ok(Self {
			raw: data,
			hash: hasher.finalize().to_vec(),
			html,
			syntax,
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

pub fn validate_title(s: &str) -> StdResult<(), String> {
	if s.contains(|c: char| "\r\n\t".contains(c)) {
		Err(String::from("the title cannot contain tabs or newlines"))
	} else if s.trim().len() < 3 {
		Err(String::from("the title must be at least 3 characters long"))
	} else {
		Ok(())
	}
}
