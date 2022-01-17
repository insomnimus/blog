pub use askama::Template;
pub use axum::{
	http::StatusCode,
	response::Html,
};
pub use log::{
	error,
	info,
};
pub use sqlx::{
	postgres::PgRow,
	query,
	query_as,
	types::chrono::{
		DateTime,
		NaiveDateTime,
		TimeZone,
		Utc,
	},
	Executor,
	Row,
};

pub use crate::{
	db::db,
	ext::*,
	response::*,
};

pub type HttpResponse<T = Html<String>> = ::std::result::Result<T, (StatusCode, &'static str)>;
