use std::path::Path;

use anyhow::Context;
use clap::crate_version;
use directories::{
	ProjectDirs,
	UserDirs,
};
use tokio::{
	fs,
	sync::OnceCell,
};

use crate::{
	article,
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
		.arg(
			arg!(-C --config [PATH] "Path to the config file.")
				.env("BLOG_CONFIG_PATH")
				.hide_env_values(true)
				.global(true),
		)
		.subcommands([article::app(), music::app(), post::app()])
}

pub async fn run() -> Result<()> {
	let m = app().get_matches();

	match m.subcommand().unwrap() {
		("article", m) => article::run(m).await,
		("post", m) => post::run(m).await,
		("music", m) => music::run(m).await,
		_ => unreachable!(),
	}
}

#[derive(serde::Deserialize, Default, Debug)]
pub struct Config {
	#[serde(rename = "cli_db_url")]
	pub db: Option<String>,
	pub sftp_uri: Option<SftpUri>,
	pub ssh_config: Option<String>,
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

impl Config {
	async fn get_or_init<P: AsRef<Path>>(path: Option<P>) -> Result<&'static Self> {
		if let Some(c) = CONFIG.get() {
			return Ok(c);
		}

		match path {
			Some(p) => {
				let data = fs::read_to_string(p.as_ref()).await?;
				let config: Self =
					toml::from_str(&data).context("failed to parse the config file")?;
				CONFIG.set(config).expect("config was already initialized");
			}
			None => {
				if let Some(proj) = ProjectDirs::from("", "", "blog") {
					match fs::read_to_string(&proj.config_dir().join("config.toml")).await {
						Err(_) => {
							CONFIG.set(Self::default()).ok();
						}
						Ok(data) => {
							let config: Self =
								toml::from_str(&data).context("failed to parse the config file")?;
							CONFIG.set(config).expect("config was already initialized");
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

	pub async fn sftp(m: &ArgMatches) -> Result<Sftp> {
		let uri = match m.value_of("sftp") {
			Some(s) => s.parse::<SftpUri>()?,
			None => Self::get_or_init(m.value_of("config"))
				.await?
				.sftp_uri
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
