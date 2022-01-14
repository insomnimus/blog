mod fetch;
mod publish;

use std::{
	fs,
	io,
	path::Path,
};

use fetch::Fetch;
use publish::Publish;
use pulldown_cmark::{
	html::push_html,
	Options,
	Parser,
};
use sha2::{
	Digest,
	Sha256,
};

use crate::prelude::*;

pub enum ArticleCmd {
	Fetch(Fetch),
	Publish(Publish),
}

impl ArticleCmd {
	pub fn app() -> App<'static> {
		App::new("article")
			.about("Manage articles.")
			.setting(AppSettings::SubcommandRequiredElseHelp)
			.arg(
				arg!(-X --database <URL> "Database URL.")
					.global(true)
					.env("BLOG_DB_URL")
					.hide_env_values(true),
			)
			.subcommands([Fetch::app(), Publish::app()])
	}

	pub fn from_matches(m: &ArgMatches) -> Self {
		block!(async move { init_db(m.value_of("database").unwrap()).await }).unwrap_or_else(|e| {
			eprintln!("error: {}", e);
			std::process::exit(1);
		});
		match m.subcommand().unwrap() {
			("fetch", m) => Self::Fetch(Fetch::from_matches(m)),
			("publish", m) => Self::Publish(Publish::from_matches(m)),
			_ => unreachable!(),
		}
	}

	pub fn run(self) -> Result<()> {
		match self {
			Self::Fetch(x) => x.run(),
			Self::Publish(x) => x.run(),
		}
	}
}

struct Article {
	title: String,
	published: DateTime<Utc>,
	updated: Option<DateTime<Utc>>,
	contents: ArticleContents,
}

struct ArticleContents {
	markdown: String,
	html: String,
	hash: Vec<u8>,
}

impl ArticleContents {
	fn read_from_file<P: AsRef<Path>>(p: P) -> io::Result<Self> {
		let data = fs::read_to_string(p.as_ref())?;
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
