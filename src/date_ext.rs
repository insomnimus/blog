use sqlx::types::chrono::{
	DateTime,
	Utc,
};

pub trait DateTimeExt: Sized {
	type Output;
	fn format_utc(&self) -> Self::Output;
}

impl DateTimeExt for DateTime<Utc> {
	type Output = String;

	fn format_utc(&self) -> String {
		self.format("%Y-%m-%dT%H:%MZ").to_string()
	}
}

impl DateTimeExt for Option<DateTime<Utc>> {
	type Output = Option<String>;

	fn format_utc(&self) -> Option<String> {
		self.as_ref().map(|d| d.format_utc())
	}
}
