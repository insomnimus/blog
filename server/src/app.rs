use clap::{
	arg,
	Command,
};

pub struct Config {
	pub site_name: String,
	pub description: String,
	pub db_url: String,
	pub listen: String,
	pub media_dir: String,
	pub copyright: String,
	pub url: Option<String>,
}

impl Config {
	pub fn from_args() -> Self {
		let m = Command::new("blog")
			.about("The blog webserver.")
			.args(&[
			arg!(--name <NAME> "The site name in human readable form.")
			.env("BLOG_SITE_NAME"),
			arg!(--description <DESCRIPTION> "A very brief description of the website like 'Blog and short posts about programming'.")
			.env("BLOG_SITE_DESCRIPTION"),
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
				arg!(--url [URL] "The URL of this website, including the protocol. Used in the Atom feed.")
				.env("BLOG_URL"),
			])
			.get_matches();

		Self {
			site_name: m.value_of("name").unwrap().into(),
			description: m.value_of("description").unwrap().into(),
			db_url: m.value_of("database").unwrap().into(),
			listen: m.value_of("listen").unwrap().into(),
			media_dir: m.value_of("media-dir").unwrap().into(),
			copyright: m.value_of("copyright").unwrap_or_default().into(),
			url: m
				.value_of("url")
				.map(|s| s.trim_end_matches('/').to_string()),
		}
	}
}
