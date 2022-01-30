use std::str::FromStr;

use serde::Serialize;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Format {
	Human,
	Json,
	Tsv,
}

impl FromStr for Format {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.eq_ignore_ascii_case("human") {
			Ok(Self::Human)
		} else if s.eq_ignore_ascii_case("json") {
			Ok(Self::Json)
		} else if s.eq_ignore_ascii_case("tsv") {
			Ok(Self::Tsv)
		} else {
			Err("value must be one of human, json or tsv")
		}
	}
}

impl Format {
	pub const VALUES: &'static [&'static str] = &["human", "json", "tsv"];

	pub fn print<T: Formattable>(self, val: &T) -> anyhow::Result<()> {
		val.print(self)
	}
}

pub trait Formattable: Tsv + Serialize {
	fn human(&self) -> String;

	fn print(&self, format: Format) -> anyhow::Result<()> {
		let s = match format {
			Format::Human => self.human(),
			Format::Tsv => self.tsv(),
			Format::Json => serde_json::to_string(self)?,
		};
		println!("{}", s);
		Ok(())
	}
}

pub trait Tsv {
	fn tsv(&self) -> String;
}

impl Tsv for str {
	fn tsv(&self) -> String {
		let mut buf = String::new();
		for c in self.chars() {
			match c {
				'\t' => buf.push_str("\\t"),
				'\r' => buf.push_str("\\r"),
				'\n' => buf.push_str("\\n"),
				'\\' => buf.push_str("\\\\"),
				_ => buf.push(c),
			};
		}
		buf
	}
}

impl<'a> Tsv for &'a str {
	fn tsv(&self) -> String {
		str::tsv(self)
	}
}

impl Tsv for String {
	fn tsv(&self) -> String {
		str::tsv(self.as_str())
	}
}

impl<T: Tsv> Tsv for Option<T> {
	fn tsv(&self) -> String {
		self.as_ref().map(|x| x.tsv()).unwrap_or_default()
	}
}

impl<T: Tsv> Tsv for Vec<T> {
	fn tsv(&self) -> String {
		let mut buf = String::from("[");
		for (i, x) in self.iter().enumerate() {
			if i > 0 {
				buf.push_str(", ");
			}
			buf.push_str(&x.tsv());
		}

		buf.push(']');
		buf
	}
}
