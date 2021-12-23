mod app;
mod home;
mod post;
mod prelude;

use std::{
	env,
	path::PathBuf,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let posts = PathBuf::from(env::var("POSTS_DIR")?);
	let cache = PathBuf::from(env::var("CACHE_DIR")?);

	let app = app::App::new(&posts, &cache)?.build();
	axum::Server::bind(&"0.0.0.0:3000".parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
