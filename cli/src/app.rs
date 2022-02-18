use std::path::Path;

use anyhow::Context;
use clap::crate_version;
use directories::ProjectDirs;
use serde::Deserialize;
use tokio::{
	fs,
	sync::OnceCell,
};
use url::Url;

use crate::{
	about,
	article,
	cmd::Cmd,
	music,
	post,
	prelude::*,
	sftp::{
		Sftp,
		SftpUri,
	},
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
		])
		.subcommands([about::app(), article::app(), music::app(), post::app()])
}

pub async fn run() -> Result<()> {
	let m = app().get_matches();

	match m.subcommand().unwrap() {
		("about", m) => about::run(m).await,
		("article", m) => article::run(m).await,
		("post", m) => post::run(m).await,
		("music", m) => music::run(m).await,
		_ => unreachable!(),
	}
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
	#[serde(rename = "db-url")]
	pub db: Option<Url>,
	pub sftp_url: Option<SftpUri>,
	pub sftp_command: Option<Cmd>,
	#[serde(default)]
	pub hooks: Hooks,
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "kebab-case", default)]
pub struct Hooks {
	pub pre_db: Option<Cmd>,
	pub pre_sftp: Option<Cmd>,
}

#[derive(Deserialize, Debug, Default)]
struct AppConfig {
	#[serde(default)]
	cli: Config,
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

impl Config {
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
				.as_ref()
				.map(|x| x.as_str())
				.ok_or_else(|| anyhow!("the database url is missing")),
		}
	}

	pub async fn sftp(m: &ArgMatches) -> Result<Sftp> {
		let uri = match m.value_of("sftp") {
			Some(s) => s.parse::<SftpUri>()?,
			None => Self::get_or_init(m.value_of("config"))
				.await?
				.sftp_url
				.clone()
				.ok_or_else(|| anyhow!("missing sftp uri"))?,
		};

		let cmd = match m.value_of_t::<Cmd>("sftp-command") {
			Ok(c) => c,
			Err(_) => Self::get_or_init(m.value_of("config"))
				.await?
				.sftp_command
				.clone()
				.unwrap_or_else(|| Cmd {
					cmd: "sftp".into(),
					args: vec!["-b".to_string(), "-".to_string()],
				}),
		};

		Ok(Sftp { uri, cmd })
	}
}
