use std::{
	path::{
		Path,
		PathBuf,
	},
	sync::atomic::Ordering,
};

use clap::crate_version;
use config::{
	ConfigError,
	Environment,
	File,
	FileFormat,
};
use directories::ProjectDirs;
use serde::Deserialize;
use tokio::sync::OnceCell;

use crate::{
	about,
	article,
	cmd::Cmd,
	gc,
	media,
	music,
	note,
	prelude::*,
	render,
};

pub fn app() -> App {
	App::new("blog")
		.about("Blog management cli.")
		.version(crate_version!())
		.subcommand_required(true)
		.arg_required_else_help(true)
		.propagate_version(true)
		.infer_subcommands(true)
		.args(&[
			arg!(-C --config [PATH] "Path to the config file.").env("BLOG_CONFIG_PATH"),
			arg!(-D --database [URL] "Database URL.")
				.env("BLOG_CLI_DB_URL")
				.hide_env_values(true),
			arg!(-M --"media-dir" [DIR] "The path of the media directory for the attachments.")
				.env("BLOG_CLI_MEDIA_DIR"),
		])
		.subcommands([
			about::app(),
			article::app(),
			gc::app(),
			music::app(),
			note::app(),
			render::app(),
		])
}

pub async fn run() -> Result<()> {
	let m = app().get_matches();
	task::block_in_place(|| Config::init(&m))?;

	if m.subcommand_name() != Some("render") {
		let db = Config::database()?;
		run_hook!(pre_db).await?;
		init_db(db).await?;
	}

	match m.subcommand().unwrap() {
		("about", m) => about::run(m).await,
		("article", m) => article::run(m).await,
		("gc", m) => gc::run(m).await,
		("note", m) => note::run(m).await,
		("music", m) => music::run(m).await,
		("render", m) => render::run(m).await,
		_ => unreachable!(),
	}?;

	if media::ACCESSED.load(Ordering::Relaxed) {
		run_hook!(post_media).await?;
	}

	Ok(())
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
	#[serde(rename = "db-url")]
	pub db: Option<String>,
	pub media_dir: Option<PathBuf>,
	#[serde(default)]
	pub hooks: Hooks,
	pub editor: Option<Cmd>,
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "kebab-case", default)]
pub struct Hooks {
	pub pre_db: Option<Cmd>,
	pub pre_media: Option<Cmd>,
	pub post_media: Option<Cmd>,
}

#[derive(Deserialize, Debug, Default)]
struct AppConfig {
	#[serde(default)]
	cli: Config,
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

impl Config {
	pub fn get() -> &'static Self {
		CONFIG
			.get()
			.expect("Config::get called without initialization")
	}

	fn database() -> Result<&'static str> {
		CONFIG
			.get()
			.and_then(|c| c.db.as_deref())
			.ok_or_else(|| anyhow!("missing the database connection string"))
	}

	pub fn media_dir() -> Result<&'static Path> {
		CONFIG
			.get()
			.and_then(|c| c.media_dir.as_deref().map(Path::new))
			.ok_or_else(|| anyhow!("missing the media directory path"))
	}

	fn init(m: &ArgMatches) -> StdResult<&'static Self, ConfigError> {
		let mut config = config::Config::builder()
			.set_override_option("cli.db-url", m.value_of("database"))?
			.set_override_option("cli.media-dir", m.value_of("media-dir"))?
			.add_source(
				Environment::with_prefix("BLOG")
					.prefix_separator("_")
					.separator("_")
					.ignore_empty(true)
					.source(Some(normalize_env())),
			);

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
		}

		let config: AppConfig = config.build()?.try_deserialize()?;
		CONFIG.set(config.cli).unwrap();
		Ok(CONFIG.get().unwrap())
	}
}

fn normalize_env() -> std::collections::HashMap<String, String> {
	std::env::vars()
		.map(|(mut k, v)| {
			if k.to_lowercase().starts_with("blog_cli_") {
				k = format!("BLOG_CLI_{}", k["BLOG_CLI_".len()..].replace('_', "-"));
			}
			(k, v)
		})
		.collect()
}
