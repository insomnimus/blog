use clap::{
	arg,
	App,
};

pub struct Config {
	pub db_url: String,
	pub listen: String,
}

impl Config {
	pub fn from_args() -> Self {
		let m = App::new("blog")
			.about("The blog webserver.")
			.args(&[
				arg!(-d --database <URL> "The database url, must be postgresql.")
					.env("BLOG_DB_URL"),
				arg!(-l --listen [ADDRESS] "Listen on the given address.")
					.default_value("0.0.0.0:8080")
					.env("BLOG_LISTEN_ADDRESS"),
			])
			.get_matches();

		Self {
			db_url: m.value_of("database").unwrap().into(),
			listen: m.value_of("listen").unwrap().into(),
		}
	}
}
