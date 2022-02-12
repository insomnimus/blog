use std::borrow::Cow;

use sqlx::types::chrono::{
	DateTime,
	NaiveDateTime,
	Utc,
};

pub trait DateTimeExt {
	type Output;
	fn format_utc(&self) -> Self::Output;
	fn format_rss(&self) -> Self::Output;
}

impl DateTimeExt for DateTime<Utc> {
	type Output = String;

	fn format_utc(&self) -> String {
		self.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
		// self.format("%+").to_string()
		// self.format("%Y-%m-%dT%H:%MZ").to_string()
	}

	fn format_rss(&self) -> String {
		self.format("%a, %d %b %Y %T GMT").to_string()
	}
}

impl DateTimeExt for Option<DateTime<Utc>> {
	type Output = Option<String>;

	fn format_utc(&self) -> Option<String> {
		self.map(|d| d.format_utc())
	}

	fn format_rss(&self) -> Self::Output {
		self.map(|d| d.format_rss())
	}
}

impl DateTimeExt for NaiveDateTime {
	type Output = String;

	fn format_utc(&self) -> String {
		DateTime::from_utc(*self, Utc).format_utc()
	}

	fn format_rss(&self) -> String {
		DateTime::from_utc(*self, Utc).format_rss()
	}
}

impl DateTimeExt for Option<NaiveDateTime> {
	type Output = Option<String>;

	fn format_utc(&self) -> Option<String> {
		self.map(|d| DateTime::from_utc(d, Utc).format_utc())
	}

	fn format_rss(&self) -> Self::Output {
		self.map(|d| d.format_rss())
	}
}

pub trait DefaultExt {
	fn take(&mut self) -> Self;
}

impl<T: Default> DefaultExt for T {
	fn take(&mut self) -> Self {
		std::mem::take(self)
	}
}

pub trait SplitWords: AsRef<str> {
	fn first_words(&'_ self, max: usize) -> std::borrow::Cow<'_, str> {
		let s = self.as_ref().trim();
		if s.len() <= max {
			return s.into();
		}
		let mut buf = String::with_capacity(max);
		for word in s.split_whitespace() {
			if buf.len() + 4 + word.len() >= max {
				buf.truncate(max - 3);
				buf.push_str("...");
				break;
			}
			buf.push(' ');
			buf.push_str(word);
		}
		buf.into()
	}

	fn first_line_words(&'_ self, max_len: usize) -> Cow<'_, str> {
		self.as_ref()
			.trim()
			.split('\n')
			.next()
			.unwrap_or_default()
			.first_words(max_len)
	}
}

impl SplitWords for str {}
