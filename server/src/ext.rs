use axum::{
	http::StatusCode,
	response::Html,
};
use sqlx::types::chrono::{
	DateTime,
	NaiveDateTime,
	Utc,
};

pub trait DateTimeExt {
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

impl DateTimeExt for NaiveDateTime {
	type Output = String;

	fn format_utc(&self) -> String {
		DateTime::from_utc(*self, Utc).format_utc()
	}
}

impl DateTimeExt for Option<NaiveDateTime> {
	type Output = Option<String>;

	fn format_utc(&self) -> Option<String> {
		self.map(|d| DateTime::from_utc(d, Utc).format_utc())
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

pub trait ResultExt<T, E>: Sized {
	fn html(self) -> Result<Html<T>, E>;
	fn or_code(self, code: StatusCode) -> Result<T, StatusCode>;

	fn or_404(self) -> Result<T, StatusCode> {
		self.or_code(StatusCode::NOT_FOUND)
	}

	fn or_500(self) -> Result<T, StatusCode> {
		self.or_code(StatusCode::INTERNAL_SERVER_ERROR)
	}

	fn or_503(self) -> Result<T, StatusCode> {
		self.or_code(StatusCode::SERVICE_UNAVAILABLE)
	}
}

impl<T: Sized, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
	fn or_code(self, code: StatusCode) -> Result<T, StatusCode> {
		self.map_err(|e| {
			crate::prelude::error!("{}", e);
			code
		})
	}

	fn html(self) -> Result<Html<T>, E> {
		self.map(Html)
	}
}

pub trait OptionExt<T: Sized> {
	fn or_404(self) -> Result<T, StatusCode>;
	fn html(self) -> Option<Html<T>>;
}

impl<T: Sized> OptionExt<T> for Option<T> {
	fn or_404(self) -> Result<T, StatusCode> {
		self.ok_or(StatusCode::NOT_FOUND)
	}

	fn html(self) -> Option<Html<T>> {
		self.map(Html)
	}
}
