pub use std::borrow::Cow;

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
pub use futures::prelude::*;
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
pub use tokio::sync::{
	OnceCell,
	RwLock,
};

pub(crate) use crate::{
	db::db,
	ext::*,
	response::*,
	Cache,
};

pub type HttpResponse<T = Html<String>> = ::std::result::Result<T, (StatusCode, &'static str)>;
