use std::{
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
};

use axum::{
	http::StatusCode,
	routing::{
		get,
		Router,
	},
};
use indexmap::IndexMap;
use tokio::sync::RwLock;

use crate::prelude::*;

pub struct App {
	home: Home,
	posts_dir: PathBuf,
	cache_dir: PathBuf,
}

impl App {
	pub fn new(posts_dir: &Path, cache_dir: &Path) -> Result<Self> {
		let mut posts = posts_dir
			.read_dir()?
			.filter_map(|res| match res.map(|entry| entry.path()) {
				Err(e) => Some(Err(anyhow::Error::from(e))),
				Ok(p) if p.extension().map_or(false, |ext| ext.eq("md")) => {
					Some(Post::new(&p, cache_dir).map(|post| (post.url_title.clone(), post)))
				}
				_ => None,
			})
			.collect::<Result<IndexMap<_, _>, _>>()?;

		posts.sort_by(|_, a, _, b| b.date.cmp(&a.date));

		let home = Home { posts };
		Ok(Self {
			posts_dir: posts_dir.to_path_buf(),
			cache_dir: cache_dir.to_path_buf(),
			home,
		})
	}
}

impl App {
	pub fn build(self) -> Router {
		let Self { home, .. } = self;
		let home = Arc::new(RwLock::new(home));

		let home_handler = {
			let home = Arc::clone(&home);
			move || async move {
				home.read()
					.await
					.render()
					.map_err(|e| {
						error!("error rendering home: {}", e);
						StatusCode::INTERNAL_SERVER_ERROR
					})
					.map(Html)
			}
		};

		let posts_handler = {
			let home = Arc::clone(&home);
			move |path: axum::extract::Path<String>| async move {
				match home.read().await.posts.get(&path.0) {
					None => Err(StatusCode::NOT_FOUND),
					Some(p) => p
						.render()
						.await
						.map_err(|e| {
							error!("failed to render {}: {}", &p.title, e);
							StatusCode::INTERNAL_SERVER_ERROR
						})
						.map(Html),
				}
			}
		};

		Router::new()
			.route("/", get(home_handler))
			.route("/posts/:post", get(posts_handler))
	}
}
