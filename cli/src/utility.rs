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
		(music) => {
		sqlx::query!("INSERT INTO cache(_instance)
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE SET
		home = CURRENT_TIMESTAMP,
		music = CURRENT_TIMESTAMP")
	};
	(all) => {
		sqlx::query!("INSERT INTO cache(_instance) 
		VALUES('TRUE')
		ON CONFLICT(_instance) DO UPDATE SET
		home = CURRENT_TIMESTAMP,
		articles = CURRENT_TIMESTAMP,
		posts = CURRENT_TIMESTAMP,
		music = CURRENT_TIMESTAMP")
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

macro_rules! run_hook {
	($hook:ident, $matches:expr) => {
		async {
			let conf = $crate::app::Config::get_or_init($matches.value_of("config")).await?;
			if let Some(hook) = &conf.hooks.$hook {
				tokio::task::block_in_place(|| hook.to_std().status())?;
			}
			Ok::<_, anyhow::Error>(())
		}
	};
}

pub(crate) use clear;
pub(crate) use confirm;
pub(crate) use run_hook;

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

pub fn rand_filename() -> String {
	use rand::{
		distributions::Alphanumeric,
		Rng,
	};
	rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(7)
		.map(char::from)
		.collect()
}
