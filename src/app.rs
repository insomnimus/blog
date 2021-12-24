use std::sync::Arc;

use axum::{
	http::StatusCode,
	routing::{
		get,
		Router,
	},
};
use notify::Watcher;
use tokio::{
	sync::{
		mpsc,
		RwLock,
	},
	task,
};

use crate::prelude::*;

impl Home {
	pub async fn build_app(self) -> Result<Router> {
		let home = Arc::new(RwLock::new(self));
		start_watching(Arc::clone(&home)).await?;

		let home_handler = {
			let home = Arc::clone(&home);
			move || async move { home.read().await.render().or_503().html() }
		};

		let posts_handler = {
			// let home = Arc::clone(&home);
			move |path: axum::extract::Path<String>| async move {
				match home.read().await.posts.get(&path.0) {
					None => Err(StatusCode::NOT_FOUND),
					Some(p) => p.render().await.or_503().html(),
				}
			}
		};

		Ok(Router::new()
			.route("/", get(home_handler))
			.route("/posts/:post", get(posts_handler)))
	}
}

async fn start_watching(home: Arc<RwLock<Home>>) -> Result<()> {
	use notify::{
		event::EventKind,
		RecommendedWatcher,
		RecursiveMode,
	};

	let (tx, mut rx) = mpsc::unbounded_channel();

	let mut watcher = RecommendedWatcher::new(move |res| match res {
		Ok(ev) => tx.send(ev).unwrap(),
		Err(e) => error!("watch: {}", e),
	})?;
	let posts_dir = home.read().await.posts_dir.clone();
	watcher.watch(&posts_dir, RecursiveMode::Recursive)?;
	std::mem::forget(watcher);

	/*
	task::spawn_blocking({
		let posts_dir = home.read().await.posts_dir.clone();
		move || {
			watcher.watch(&posts_dir, RecursiveMode::Recursive)
		}
	});
	*/

	task::spawn(async move {
		while let Some(ev) = rx.recv().await {
			if matches!(
				ev.kind,
				EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
			) {
				let mut home = home.write().await;
				if let Err(e) = home.reload().await {
					error!("failed reloading files: {}", e);
				}
			}
		}
		info!("the watcher is dropped");
	});
	Ok(())
}
