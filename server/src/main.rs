mod app;
mod article;
mod db;
mod ext;
mod home;
mod media;
mod music;
mod post;
mod prelude;
mod response;
mod search;

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
use sqlx::types::chrono::{
	NaiveDateTime,
	Utc,
};
use tokio::sync::{
	OnceCell,
	RwLock,
};
use tower::{
	timeout::TimeoutLayer,
	ServiceBuilder,
};
use tower_http::services::ServeDir;

async fn robots_txt() -> &'static str {
	"User-agent: *
Disallow: /api/"
}

pub struct CacheData<T> {
	pub time: NaiveDateTime,
	pub data: T,
}

impl<T: Default> Default for CacheData<T> {
	fn default() -> Self {
		Self {
			time: Utc::now().naive_utc(),
			data: T::default(),
		}
	}
}

pub type Cache<T = String> = OnceCell<RwLock<CacheData<T>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	if env::var_os("RUST_LOG").is_none() {
		env::set_var("RUST_LOG", "blog=debug,tower_http=debug")
	}
	tracing_subscriber::fmt::init();

	let config = app::Config::from_args();
	db::init(&config.db_url).await?;

	let static_handler =
		get_service(ServeDir::new("static")).handle_error(|e: std::io::Error| async move {
			log::error!("static: {e}");
			(StatusCode::NOT_FOUND, "The requested file is not found.")
		});

	let media_handler = get_service(ServeDir::new(&config.media_dir)).handle_error(
		|e: std::io::Error| async move {
			log::error!("media: {e}");
			(StatusCode::NOT_FOUND, "The requested file is not found.")
		},
	);

	let api = Router::new().route("/posts", get(post::handle_api));

	let app = Router::new()
		.nest("/media", media_handler)
		.nest("/api", api)
		.nest("/static", static_handler)
		.route("/", get(home::handle_home))
		.route("/posts", get(post::handle_posts))
		.route("/posts/:id", get(post::handle_post))
		.route("/articles", get(article::handle_articles))
		.route("/articles/:article", get(article::handle_article))
		.route("/search", get(search::handle_search))
		.route("/music/:id", get(music::handle_music))
		.route("/music", get(music::handle_music_page))
		.route("/robots.txt", get(robots_txt))
		.layer(
			ServiceBuilder::new()
			// this middleware goes above `TimeoutLayer` because it will receive
						// errors returned by `TimeoutLayer
						.layer(HandleErrorLayer::new(|_: BoxError| async {
							StatusCode::REQUEST_TIMEOUT
							}))
							.layer(TimeoutLayer::new(Duration::from_secs(10))),
		);

	axum::Server::bind(&config.listen.parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
