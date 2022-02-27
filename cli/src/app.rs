use std::{
	path::{
		Path,
		PathBuf,
	},
	sync::atomic::Ordering,
};

use anyhow::Context;
use clap::crate_version;
use directories::ProjectDirs;
use serde::Deserialize;
use tokio::{
	fs,
	sync::OnceCell,
};

use crate::{
	about,
	article,
	cmd::Cmd,
	gc,
	media,
	music,
	post,
	prelude::*,
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
			arg!(-C --config [PATH] "Path to the config file.")
				.env("BLOG_CONFIG_PATH")
				.global(true),
			arg!(-D --database [URL] "Database URL.")
				.global(true)
				.env("BLOG_DB_URL")
				.hide_env_values(true),
			arg!(-M --"media-dir" [DIR] "The path of the media directory for the attachments.")
				.env("BLOG_MEDIA_DIR")
				.global(true),
		])
		.subcommands([
			about::app(),
			article::app(),
			gc::app(),
			music::app(),
			post::app(),
		])
}

pub async fn run() -> Result<()> {
	let m = app().get_matches();
	let db = Config::database(&m).await?;
	run_hook!(pre_db, m).await?;
	init_db(db).await?;

	match m.subcommand().unwrap() {
		("about", m) => about::run(m).await,
		("article", m) => article::run(m).await,
		("gc", m) => gc::run(m).await,
		("post", m) => post::run(m).await,
		("music", m) => music::run(m).await,
		_ => unreachable!(),
	}?;

	if media::ACCESSED.load(Ordering::Relaxed) {
		run_hook!(post_media, m).await?;
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
	pub fn try_get() -> Option<&'static Self> {
		CONFIG.get()
	}

	pub async fn get_or_init<P: AsRef<Path>>(path: Option<P>) -> Result<&'static Self> {
		if let Some(c) = CONFIG.get() {
			return Ok(c);
		}

		match path {
			Some(p) => {
				let data = fs::read_to_string(p.as_ref()).await?;
				let config: AppConfig =
					toml::from_str(&data).context("failed to parse the config file")?;
				CONFIG
					.set(config.cli)
					.expect("config was already initialized");
			}
			None => {
				if let Some(proj) = ProjectDirs::from("", "", "blog") {
					match fs::read_to_string(&proj.config_dir().join("config.toml")).await {
						Err(_) => {
							CONFIG.set(Self::default()).ok();
						}
						Ok(data) => {
							let config: AppConfig =
								toml::from_str(&data).context("failed to parse the config file")?;
							CONFIG
								.set(config.cli)
								.expect("config was already initialized");
						}
					}
				}
			}
		};

		Ok(CONFIG.get().unwrap())
	}

	pub async fn database(m: &ArgMatches) -> Result<&str> {
		match m.value_of("database") {
			Some(db) => Ok(db),
			None => Self::get_or_init(m.value_of("config"))
				.await?
				.db
				.as_deref()
				.ok_or_else(|| anyhow!("the database url is missing")),
		}
	}

	pub async fn media_dir(m: &ArgMatches) -> Result<&Path> {
		match m.value_of("media-dir") {
			Some(x) => Ok(Path::new(x)),
			None => {
				let c = Self::get_or_init(m.value_of("config")).await?;
				c.media_dir
					.as_deref()
					.ok_or_else(|| anyhow!("media-dir is required but not specified"))
			}
		}
	}
}
