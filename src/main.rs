mod app;
mod home;
mod post;
mod prelude;
mod response_ext;

use std::{
	env,
	path::PathBuf,
	time::Duration,
};

use axum::{
	error_handling::HandleErrorLayer,
	http::StatusCode,
	routing::{
		get_service,
		Router,
	},
	BoxError,
};
use tower::{
	timeout::TimeoutLayer,
	ServiceBuilder,
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let posts = PathBuf::from(env::var("POSTS_DIR")?);
	let cache = PathBuf::from(env::var("CACHE_DIR")?);

	if env::var_os("RUST_LOG").is_none() {
		env::set_var("RUST_LOG", "blog=debug,tower_http=debug")
	}
	tracing_subscriber::fmt::init();

	let static_handler =
		get_service(ServeDir::new("static")).handle_error(|error: std::io::Error| async move {
			eprintln!("static: {}", error);
			(StatusCode::INTERNAL_SERVER_ERROR, "page not found")
		});

	let app = home::Home::new(&posts, &cache)?
		.build_app()
		.await?
		.nest("/static", static_handler)
		.layer(
			ServiceBuilder::new()
            // this middleware goes above `TimeoutLayer` because it will receive
            // errors returned by `TimeoutLayer`
            .layer(HandleErrorLayer::new(|_: BoxError| async {
                StatusCode::REQUEST_TIMEOUT
            }))
            .layer(TimeoutLayer::new(Duration::from_secs(10))),
		);

	/*
			let app = Router::new()
			.nest("/static", static_handler)
			.nest("/", posts);
	*/
	axum::Server::bind(&"0.0.0.0:3000".parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
