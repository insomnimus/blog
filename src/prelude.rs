pub use std::mem;

// pub use futures::prelude::*;
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
	date_ext::*,
	db::prelude::*,
	response_ext::*,
};

pub type HtmlResponse<T = Html<String>> = ::std::result::Result<T, StatusCode>;
