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
			arg!(-X --database <URL> "Database URL.")
				.env("BLOG_DB_URL")
				.hide_env_values(true),
		)
		.subcommands([fetch::app(), list::app(), publish::app()])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	init_db(m.value_of("database").unwrap()).await?;

	match m.subcommand().unwrap() {
		("fetch", m) => fetch::run(m).await,
		("list", m) => list::run(m).await,
		("publish", m) => publish::run(m).await,
		_ => unreachable!(),
	}
}

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
	url_title: String,
	published: String,
	updated: Option<String>,
	tags: Vec<String>,
}

impl Formattable for ArticleInfo {
	fn human(&self) -> String {
		format!("{} ({})", &self.title, &self.published,)
	}
}

impl Tsv for ArticleInfo {
	fn tsv(&self) -> String {
		format!(
			"{title}\t{published}\t{updated}\t{tags}\t{url_title}",
			title = self.title.tsv(),
			published = &self.published,
			updated = self.updated.as_deref().tsv(),
			tags = self.tags.tsv(),
			url_title = self.url_title.tsv(),
		)
	}
}
