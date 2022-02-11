use clap::{
	arg,
	App,
};

pub struct Config {
	pub db_url: String,
	pub listen: String,
	pub media_dir: String,
	pub copyright: String,
}

impl Config {
	pub fn from_args() -> Self {
		let m = App::new("blog")
			.about("The blog webserver.")
			.args(&[
				arg!(-d --database <URL> "The database url, must be postgresql.")
					.env("BLOG_SERVER_DB_URL"),
					arg!(--copyright [NAME] "The copyright holder name.")
					.env("BLOG_COPYRIGHT_NAME"),
				arg!(-l --listen [ADDRESS] "Listen on the given address.")
					.default_value("0.0.0.0:8080")
					.env("BLOG_LISTEN_ADDRESS"),
					arg!(-m --"media-dir" [MEDIA_DIR] "The media directory that will be served on /media.")
					.default_value("media")
					.env("BLOG_MEDIA_DIR"),
			])
			.get_matches();

		Self {
			db_url: m.value_of("database").unwrap().into(),
			listen: m.value_of("listen").unwrap().into(),
			media_dir: m.value_of("media-dir").unwrap().into(),
			copyright: m.value_of("copyright").unwrap_or_default().into(),
		}
	}
}
