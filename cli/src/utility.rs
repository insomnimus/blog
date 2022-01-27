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

		println!("{} [y/n]: ", format_args!($fmt, $($args),*));
		stdout.flush().unwrap();
		let stdin = stdin.lock();
		let v = stdin.lines().next().unwrap().map(|s| s.eq_ignore_ascii_case("y") || s.eq_ignore_ascii_case("yes"));
		v
	}};
}

pub(crate) use clear;
pub(crate) use confirm;
