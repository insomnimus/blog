use std::path::Path;

use anyhow::Context;
use clap::crate_version;
use directories::{
	ProjectDirs,
	UserDirs,
};
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

pub fn app() -> App<'static> {
	App::new("blog")
		.about("Blog management cli.")
		.version(crate_version!())
		.setting(AppSettings::SubcommandRequiredElseHelp)
		.global_setting(AppSettings::InferSubcommands)
		.global_setting(AppSettings::PropagateVersion)
		.args(&[
			arg!(-C --config [PATH] "Path to the config file.")
				.env("BLOG_CONFIG_PATH")
				.hide_env_values(true)
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
	pub ssh_config: Option<String>,
	#[serde(default)]
	pub hooks: Hooks,
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "kebab-case", default)]
pub struct Hooks {
	pub pre_db: Option<Cmd>,
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

		let ssh_config = match m.value_of("ssh-config") {
			Some(p) => Some(p.to_string()),
			None => Self::get_or_init(m.value_of("config"))
				.await?
				.ssh_config
				.clone()
				.or_else(|| {
					UserDirs::new().and_then(|d| {
						let p = d.home_dir().join(".ssh/config");
						if p.is_file() {
							Some(p.to_string_lossy().into_owned())
						} else {
							None
						}
					})
				}),
		};

		let extra_args = m
			.values_of("sftp-args")
			.into_iter()
			.flatten()
			.map(String::from)
			.collect::<Vec<_>>();

		Ok(Sftp {
			uri,
			cmd_path: "sftp".into(),
			extra_args,
			ssh_config,
		})
	}
}
