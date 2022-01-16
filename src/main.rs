mod app;
mod article;
mod date_ext;
mod db;
mod home;
mod prelude;
mod response_ext;

use std::{
	env,
	time::Duration,
};

use axum::{
	error_handling::HandleErrorLayer,
	http::StatusCode,
	routing::{
		get,
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
	if env::var_os("RUST_LOG").is_none() {
		env::set_var("RUST_LOG", "blog=debug,tower_http=debug")
	}
	tracing_subscriber::fmt::init();

	let config = app::Config::from_args();
	db::init(&config.db_url).await?;

	let static_handler =
		get_service(ServeDir::new("static")).handle_error(|error: std::io::Error| async move {
			log::error!("static: {}", error);
			(StatusCode::NOT_FOUND, "The requested file is not found.")
		});

	let app = Router::new()
		.layer(
			ServiceBuilder::new()
            // this middleware goes above `TimeoutLayer` because it will receive
            // errors returned by `TimeoutLayer`
            .layer(HandleErrorLayer::new(|_: BoxError| async {
                StatusCode::REQUEST_TIMEOUT
            }))
            .layer(TimeoutLayer::new(Duration::from_secs(10))),
		)
		.nest("/static", static_handler)
		.route("/", get(home::handle_home))
		.route("/articles/:article", get(article::handle_article));

	axum::Server::bind(&config.listen.parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
