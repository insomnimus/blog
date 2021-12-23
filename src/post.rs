use std::{
	path::{
		Path,
		PathBuf,
	},
	sync::atomic::{
		AtomicBool,
		Ordering,
	},
};

use pulldown_cmark::{
	html,
	Options,
	Parser,
};
use tokio::fs;

use crate::prelude::*;

pub struct Post {
	path: PathBuf,
	cached_path: PathBuf,
	cached: AtomicBool,
	pub title: String,
	pub url_title: String,
	pub date: DateTime<Utc>,
}

impl Post {
	pub fn new(path: &Path, cache_dir: &Path) -> Result<Self> {
		let file_stem = path
			.file_stem()
			.ok_or_else(|| anyhow!("the path does not have a base name"))?;

		let file_stem_str = file_stem.to_string_lossy();
		let (title, date) = split_title_date(&file_stem_str)?;

		let title = title.to_string();
		let mut cached_path = cache_dir.join(file_stem);
		cached_path.set_extension("html");
		let url_title = title.replace(' ', "_");
		Ok(Self {
			date,
			title,
			url_title,
			cached: AtomicBool::new(false),
			path: path.to_path_buf(),
			cached_path,
		})
	}

	pub async fn render(&self) -> Result<String> {
		let body = if self.cached.load(Ordering::Relaxed) {
			fs::read_to_string(&self.cached_path).await?
		} else {
			let md = fs::read_to_string(&self.path).await?;
			let opts = Options::all();
			let mut buf = String::new();
			html::push_html(&mut buf, Parser::new_ext(&md, opts));
			fs::write(&self.cached_path, &buf).await?;
			self.cached.store(true, Ordering::SeqCst);
			buf
		};
		PostBody {
			title: self.title.as_str(),
			date: self.date,
			html: body.as_str(),
		}
		.render()
		.map_err(|e| e.into())
	}
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostBody<'a, 'b> {
	title: &'a str,
	date: DateTime<Utc>,
	html: &'b str,
}

fn split_title_date(s: &str) -> Result<(&str, DateTime<Utc>)> {
	let (title, date) = s
		.rsplit_once(|c: char| c.is_whitespace())
		.ok_or_else(|| anyhow!("the file name contains no spaces so can't be split"))?;
	let title = title.trim();
	if title.is_empty() {
		anyhow::bail!("title is empty");
	}
	Utc.datetime_from_str(date, "%Y-%m-%d-%H-%M")
		.map(|dt| (title, dt))
		.map_err(|e| e.into())
}
