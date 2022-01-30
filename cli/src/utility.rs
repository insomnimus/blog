use clap::ArgMatches;

use crate::sftp::{
	Sftp,
	SftpCommand,
	SftpUri,
};

macro_rules! clear {
	(home) => {
		sqlx::query!("INSERT INTO html_cache(_instance)
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE
		SET home_page = NULL")
	};
	(articles) => {
		sqlx::query!("INSERT INTO html_cache(_instance)
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE SET
		home_page = NULL,
		articles_page = NULL")
	};
	(posts) => {
		sqlx::query!("INSERT INTO html_cache(_instance)
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE SET
		home_page = NULL,
		posts_page = NULL")
	};
	(all) => {
		sqlx::query!("INSERT INTO html_cache(_instance) 
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE SET
		home_page = NULL,
		articles_page = NULL,
		posts_page = NULL")
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

pub fn sftp_args(m: &ArgMatches) -> Sftp {
	let extra_args = m
		.values_of("sftp-args")
		.into_iter()
		.flatten()
		.map(String::from)
		.collect::<Vec<_>>();

	let SftpUri { remote, root } = m.value_of_t_or_exit::<SftpUri>("sftp");

	Sftp {
		root,
		cmd: SftpCommand {
			remote,
			extra_args,
			cmd_path: "sftp".into(),
		},
	}
}

pub fn validate_sftp_uri(s: &str) -> Result<(), String> {
	s.parse::<SftpUri>()
		.map(|_| {})
		.map_err(|e| format!("the uri syntax is invalid: {e}"))
}
