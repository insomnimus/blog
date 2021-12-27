use std::path::{
	Path,
	PathBuf,
};

use indexmap::IndexMap;
use tokio::fs;

use crate::prelude::*;

#[derive(Template)]
#[template(path = "home.html")]
pub struct Home {
	pub posts: IndexMap<String, Post>,
	pub posts_dir: PathBuf,
	pub cache_dir: PathBuf,
}

impl Home {
	pub fn new(posts_dir: &Path, cache_dir: &Path) -> Result<Self> {
		let mut posts = posts_dir
			.read_dir()?
			.filter_map(|res| match res.map(|entry| entry.path()) {
				Err(e) => Some(Err(Error::from(e))),
				Ok(p) if p.extension().map_or(false, |ext| ext.eq("md")) => Some(
					Post::new(&p, cache_dir).map(|post| (post.metadata.url_title.clone(), post)),
				),
				_ => None,
			})
			.collect::<Result<IndexMap<_, _>, _>>()?;

		posts.sort_by(|_, a, _, b| b.metadata.cmp_dates(&a.metadata));

		Ok(Self {
			posts,
			posts_dir: posts_dir.to_path_buf(),
			cache_dir: cache_dir.to_path_buf(),
		})
	}

	pub async fn reload(&mut self) -> Result<()> {
		let mut posts = fs::read_dir(&self.posts_dir).await?;

		self.posts.clear();
		while let Some(res) = posts.next_entry().await.transpose() {
			match res.map(|entry| entry.path()) {
				Err(e) => error!("{}", e),
				Ok(p) if p.extension().map_or(false, |ext| ext.eq("md")) => {
					match Post::new(&p, &self.cache_dir) {
						Ok(post) => {
							self.posts.insert(post.metadata.url_title.clone(), post);
						}
						Err(e) => error!("{}", e),
					};
				}
				_ => (),
			};
		}

		self.posts
			.sort_by(|_, a, _, b| b.metadata.cmp_dates(&a.metadata));
		Ok(())
	}
}

impl Home {
	fn recent(&'_ self, n: usize) -> impl IntoIterator<Item = &'_ Post> {
		self.posts.values().take(n)
	}
}
