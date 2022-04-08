use clap::{
	arg,
	crate_version,
	Command,
};
use config::{
	ConfigError,
	Environment,
	File,
	FileFormat,
};
use directories::ProjectDirs;
use serde::{
	Deserialize,
	Serialize,
};

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
	pub site_name: String,
	pub description: String,
	pub db_url: String,
	pub listen: String,
	pub media_dir: String,
	#[serde(rename = "copyright-holder")]
	pub copyright: String,
	#[serde(rename = "site-url")]
	pub url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct AppConfig {
	server: Config,
}

impl Config {
	#[cold]
	fn app() -> Command<'static> {
		Command::new("blog-server")
		.about("The blog webserver")
		.version(crate_version!())
		.args(&[
		arg!(--name [NAME] "The website name for the page titles."),
		arg!(--description [DESCRIPTION] "a brief plaintext description of the website."),
		arg!(-d --database [URL] "The postgresql database connection string."),
		arg!(--copyright [NAME] "The copyright holder name."),
		arg!(-l --listen "The address to serve on. Example: 0.0.0.0:80"),
		arg!(-m --"media-dir" [DIRECTORY] "The local media directory that will be served on /media"),
		arg!(--url [URL] "The full URL of the website, including http or https protocol prefix. Necessary for the Atom Feed."),
		arg!(-C --config [FILE] "Path of the configuration file (TOML)")
		.env("BLOG_CONFIG_PATH"),
		])
	}

	#[cold]
	pub fn parse() -> Result<Self, ConfigError> {
		let m = Self::app().get_matches();

		let mut config = config::Config::builder()
		// defaults
		.set_default("server.listen", "0.0.0.0:8080")?
		.set_default("server.media-dir", "media")?
		// cli overrides
		.set_override_option("server.listen", m.value_of("listen"))?
		.set_override_option("server.media-dir", m.value_of("media-dir"))?
		.set_override_option("server.site-name", m.value_of("name"))?
		.set_override_option("server.description", m.value_of("description"))?
		.set_override_option("server.db-url", m.value_of("database"))?
		.set_override_option("server.copyright-holder", m.value_of("copyright"))?
		.set_override_option("server.site-url", m.value_of("url"))?
		// env
		.add_source(Environment::with_prefix("BLOG").prefix_separator("_").separator("_").ignore_empty(true).source(Some(normalize_env())));

		match m.value_of("config") {
			Some(p) => {
				config = config.add_source(File::new(p, FileFormat::Toml));
			}
			None => {
				if let Some(Ok(base)) = ProjectDirs::from("", "", "blog").map(|proj| {
					proj.config_dir()
						.join("config")
						.into_os_string()
						.into_string()
				}) {
					config = config.add_source(File::with_name(&base).required(false));
				}
			}
		};

		let config = config.build()?;
		let config: AppConfig = config.try_deserialize()?;
		Ok(config.server)
	}
}

#[cold]
fn normalize_env() -> std::collections::HashMap<String, String> {
	std::env::vars()
		.map(|(mut k, v)| {
			if k.to_lowercase().starts_with("blog_server_") {
				k = format!(
					"BLOG_SERVER_{}",
					k["BLOG_SERVER_".len()..].replace('_', "-")
				);
			}
			(k, v)
		})
		.collect()
}
