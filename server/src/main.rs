mod about;
mod app;
mod article;
mod db;
mod ext;
mod feed;
mod filters;
mod home;
mod media;
mod music;
mod note;
mod prelude;
mod response;
mod search;
mod xml;

use std::{
	env,
	io::Write,
};

use axum::{
	http::StatusCode,
	routing::{
		get,
		get_service,
		Router,
	},
};
use log::{
	info,
	warn,
};
use sqlx::types::chrono::{
	NaiveDateTime,
	Utc,
};
use tokio::sync::{
	OnceCell,
	RwLock,
};
use tower_http::services::ServeDir;

static COPYRIGHT: OnceCell<String> = OnceCell::const_new();
static SITE_URL: OnceCell<String> = OnceCell::const_new();

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
	if env::var_os("BLOG_LOG").is_none() {
		env::set_var("BLOG_LOG", "error,blog=info,tower_http=warn,sqlx=warn")
	}
	env_logger::Builder::from_env("BLOG_LOG")
		.format(|buf, record| {
			writeln!(
				buf,
				"[{level} {src}] {msg} | {ts}",
				ts = chrono::Utc::now().format("%Y-%m-%d %R"),
				level = record.level(),
				msg = record.args(),
				src = record.target(),
			)
		})
		.init();

	let config = app::Config::from_args();
	COPYRIGHT.set(config.copyright.clone()).unwrap();

	db::init(&config.db_url).await?;
	info!("connected to the database");

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

	let api = Router::new().route("/notes", get(note::handle_api));

	let app = Router::new()
		.nest("/media", media_handler)
		.nest("/api", api)
		.nest("/static", static_handler)
		.route("/", get(home::handle_home))
		.route("/notes", get(note::handle_notes))
		.route("/notes/:id", get(note::handle_note))
		.route("/articles", get(article::handle_articles))
		.route("/articles/:article", get(article::handle_article))
		.route("/search", get(search::handle_search))
		.route("/music/:id", get(music::handle_music))
		.route("/music", get(music::handle_music_page))
		.route("/about", get(about::handle_about))
		.route("/robots.txt", get(robots_txt));

	let app = match &config.url {
		None => {
			warn!("no site url is set, the atom feed won't be available");
			app
		}
		Some(url) => {
			SITE_URL.set(url.clone()).unwrap();
			app.route("/feed", get(feed::handle_feed))
		}
	};

	info!("listening on {}", &config.listen);

	axum::Server::bind(&config.listen.parse()?)
		.serve(app.into_make_service())
		.await?;

	Ok(())
}
