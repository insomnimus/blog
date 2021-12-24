mod app;
mod home;
mod post;
mod prelude;

use std::{
	env,
	path::PathBuf,
	time::Duration,
};

use axum::{
	error_handling::HandleErrorLayer,
	http::StatusCode,
	BoxError,
};
use tower::{
	timeout::TimeoutLayer,
	ServiceBuilder,
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let posts = PathBuf::from(env::var("POSTS_DIR")?);
	let cache = PathBuf::from(env::var("CACHE_DIR")?);

	let app = home::Home::new(&posts, &cache)?.build_app().await?.layer(
		ServiceBuilder::new()
            // this middleware goes above `TimeoutLayer` because it will receive
            // errors returned by `TimeoutLayer`
            .layer(HandleErrorLayer::new(|_: BoxError| async {
                StatusCode::REQUEST_TIMEOUT
            }))
            .layer(TimeoutLayer::new(Duration::from_secs(10))),
	);

	axum::Server::bind(&"0.0.0.0:3000".parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
