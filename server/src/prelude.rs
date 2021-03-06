pub use std::borrow::Cow;

pub use anyhow::{
	anyhow,
	Result,
};
pub use askama::Template;
pub use axum::{
	extract::{
		Path,
		Query,
	},
	http::StatusCode,
	response::Html,
	Json,
};
pub use futures::{
	prelude::*,
	stream::TryStreamExt,
};
pub use log::{
	debug,
	error,
	info,
	warn,
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
pub use tokio::sync::{
	OnceCell,
	RwLock,
};

pub(crate) use crate::{
	db::db,
	ext::*,
	filters,
	response::*,
	xml::Xml,
	Cache,
};

pub type HttpResponse<T = Html<String>> = ::std::result::Result<T, (StatusCode, &'static str)>;
pub type DbResult<T> = ::std::result::Result<T, sqlx::Error>;

pub fn current_year() -> u32 {
	use chrono::Datelike;
	Utc::now().year_ce().1
}
