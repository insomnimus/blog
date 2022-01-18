pub use askama::Template;
pub use axum::{
	extract::Query,
	http::StatusCode,
	response::Html,
	Json,
};
pub use log::{
	error,
	info,
};
pub use serde::{
	Deserialize,
	Serialize,
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
