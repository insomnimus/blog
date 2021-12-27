use std::{
	path::{
		Path,
		PathBuf,
	},
	str::FromStr,
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
	pub metadata: Metadata,
}

impl Post {
	pub fn new(path: &Path, cache_dir: &Path) -> Result<Self> {
		let file_stem = path
			.file_stem()
			.ok_or_else(|| anyhow!("the path does not have a base name"))?;
		let metadata = file_stem.to_string_lossy().parse::<Metadata>()?;

		let mut cached_path = cache_dir.join(file_stem);
		cached_path.set_extension("html");
		Ok(Self {
			metadata,
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
			metadata: &self.metadata,
			html: body.as_str(),
		}
		.render()
		.map_err(|e| e.into())
	}
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostBody<'a, 'b> {
	metadata: &'a Metadata,
	html: &'b str,
}

impl<'a, 'b> PostBody<'a, 'b> {
	fn date(&self) -> String {
		self.metadata.date_str()
	}

	fn updated(&self) -> Option<String> {
		self.metadata.updated_str()
	}
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metadata {
	pub title: String,
	pub url_title: String,
	pub date: DateTime<Utc>,
	pub updated: Option<DateTime<Utc>>,
}

impl Metadata {
	pub fn cmp_dates(&self, other: &Self) -> std::cmp::Ordering {
		self.updated
			.unwrap_or(self.date)
			.cmp(&other.updated.unwrap_or(other.date))
	}

	pub fn date_str(&self) -> String {
		self.date.format("%Y-%m-%dT%H:%MZ").to_string()
	}

	pub fn updated_str(&self) -> Option<String> {
		self.updated
			.map(|d| d.format("%Y-%m-%dT%H:%MZ").to_string())
	}
}

impl FromStr for Metadata {
	type Err = crate::prelude::Error;

	fn from_str(s: &str) -> Result<Self> {
		let (title, date) = s
			.rsplit_once(|c: char| c.is_whitespace())
			.ok_or_else(|| anyhow!("the file name contains no spaces so can't be split"))?;
		let title = title.trim();
		if title.is_empty() {
			anyhow::bail!("title is empty");
		}
		let date = Utc.datetime_from_str(date, "%Y-%m-%d-%H-%M")?;
		Ok(title
			.rsplit_once(' ')
			.and_then(|(left, maybe_date)| {
				Utc.datetime_from_str(maybe_date, "%Y-%m-%d-%H-%M")
					.ok()
					.map(|original_date| Self {
						title: left.trim().into(),
						url_title: left.trim().replace(|c: char| c.is_whitespace(), "_"),
						date: original_date,
						updated: Some(date),
					})
			})
			.unwrap_or_else(|| Self {
				title: title.to_string(),
				url_title: title.replace(|c: char| c.is_whitespace(), "_"),
				date,
				updated: None,
			}))
	}
}
