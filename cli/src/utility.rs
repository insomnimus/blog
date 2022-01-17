macro_rules! clear_home {
	() => {
		sqlx::query!("TRUNCATE home_cache")
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

pub(crate) use clear_home;
pub(crate) use confirm;
