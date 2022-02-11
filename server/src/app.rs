use clap::{
	arg,
	App,
};

pub struct Config {
	pub db_url: String,
	pub listen: String,
	pub media_dir: String,
	pub copyright: String,
	pub url: Option<String>,
}

impl Config {
	pub fn from_args() -> Self {
		let m = App::new("blog")
			.about("The blog webserver.")
			.args(&[
				arg!(-d --database <URL> "The database url, must be postgresql.")
					.env("BLOG_SERVER_DB_URL")
					.validator(validate_url),
					arg!(--copyright [NAME] "The copyright holder name.")
					.env("BLOG_COPYRIGHT_NAME"),
				arg!(-l --listen [ADDRESS] "Listen on the given address.")
					.default_value("0.0.0.0:8080")
					.env("BLOG_LISTEN_ADDRESS"),
					arg!(-m --"media-dir" [MEDIA_DIR] "The media directory that will be served on /media.")
					.default_value("media")
					.env("BLOG_MEDIA_DIR"),
				arg!(--url [URL] "The URL of this website, including the protocol. Used in the RSS feed.")
				.env("BLOG_URL")
				.validator(validate_url),
			])
			.get_matches();

		Self {
			db_url: m.value_of("database").unwrap().into(),
			listen: m.value_of("listen").unwrap().into(),
			media_dir: m.value_of("media-dir").unwrap().into(),
			copyright: m.value_of("copyright").unwrap_or_default().into(),
			url: m.value_of("url").map(String::from),
		}
	}
}

fn validate_url(s: &str) -> Result<(), String> {
	s.parse::<url::Url>()
		.map_err(|e| format!("invalid url: {e}"))
		.map(|_| {})
}
