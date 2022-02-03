use clap::ArgMatches;

use crate::{
	app::Config,
	sftp::{
		Sftp,
		SftpCommand,
		SftpUri,
	},
};

macro_rules! clear {
	(home) => {
		sqlx::query!("INSERT INTO cache (_instance)
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE
		SET home = CURRENT_TIMESTAMP")
	};
	(articles) => {
		sqlx::query!("INSERT INTO cache(_instance)
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE SET
		home = CURRENT_TIMESTAMP,
		articles = CURRENT_TIMESTAMP")
	};
	(posts) => {
		sqlx::query!("INSERT INTO cache(_instance)
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE SET
		home = CURRENT_TIMESTAMP,
		posts = CURRENT_TIMESTAMP")
	};
	(all) => {
		sqlx::query!("INSERT INTO cache(_instance) 
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE SET
		home = CURRENT_TIMESTAMP,
		articles = CURRENT_TIMESTAMP,
		posts = CURRENT_TIMESTAMP")
	};
}

macro_rules! confirm{
	($fmt:literal, $($args:expr),* $(,)?) => {{
		use std::io::{BufRead, Write};
		let stdin = std::io::stdin();
		let stdout = std::io::stdout();
		let mut stdout = stdout.lock();

		print!("{} [y/n]: ", format_args!($fmt, $($args),*));
		stdout.flush().unwrap();
		let stdin = stdin.lock();
		let v = stdin.lines().next().unwrap().map(|s| s.eq_ignore_ascii_case("y") || s.eq_ignore_ascii_case("yes"));
		v
	}};
	($fmt:literal) => {
		$crate::utility::confirm!($fmt,)
	};
}

pub(crate) use clear;
pub(crate) use confirm;

pub async fn sftp_args(m: &ArgMatches) -> anyhow::Result<Sftp> {
	let extra_args = m
		.values_of("sftp-args")
		.into_iter()
		.flatten()
		.map(String::from)
		.collect::<Vec<_>>();

	let SftpUri { remote, root } = Config::sftp(m).await?;

	Ok(Sftp {
		root,
		cmd: SftpCommand {
			remote,
			extra_args,
			cmd_path: "sftp".into(),
		},
	})
}

pub fn validate_sftp_uri(s: &str) -> Result<(), String> {
	s.parse::<SftpUri>()
		.map(|_| {})
		.map_err(|e| format!("the uri syntax is invalid: {e}"))
}

pub async fn edit_buf(prefix: &str, ext: &str, buf: &str) -> std::io::Result<Option<String>> {
	let mut b = edit::Builder::new();
	b.prefix(prefix).rand_bytes(4).suffix(ext);

	let edited = tokio::task::block_in_place(move || {
		println!("waiting for you to finish editing");
		edit::edit_with_builder(buf, &b)
	})?;

	let trimmed = edited.trim();
	if trimmed.is_empty() || trimmed == buf.trim() {
		Ok(None)
	} else {
		Ok(Some(edited))
	}
}

pub fn format_filename(s: &str) -> String {
	s.chars()
		.map(|c| {
			if c.is_whitespace() || c == '_' {
				'-'
			} else {
				c
			}
		})
		.filter(|&c| c.is_alphanumeric() || c == '-' || c == '.')
		.collect()
}
