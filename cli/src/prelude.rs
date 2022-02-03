use std::str::FromStr;

pub use anyhow::{
	anyhow,
	Result,
};
pub use clap::{
	arg,
	App,
	AppSettings,
	Arg,
	ArgGroup,
	ArgMatches,
};
pub use futures::prelude::*;
pub use serde::Serialize;
use sqlx::{
	postgres::PgPoolOptions as Opts,
	PgPool as Pool,
};
pub use sqlx::{
	query,
	query_as,
	types::chrono::{
		DateTime,
		Utc,
	},
};
use tokio::sync::OnceCell;

pub(crate) use crate::{
	app::Config,
	display::*,
	ext::*,
	render::*,
	utility::*,
};

pub type StdResult<T, E> = ::std::result::Result<T, E>;

static DB: OnceCell<Pool> = OnceCell::const_new();

pub fn encode_url_title(s: &str) -> String {
	let s = s
		.replace(|c: char| c.is_whitespace() || c == '_', "-")
		.to_lowercase();
	url_escape::encode_path(&s).to_string()
}

pub fn db() -> &'static Pool {
	DB.get().expect("DB is not initialized!")
}

pub async fn init_db(url: &str) -> Result<&'static Pool> {
	let pool = Opts::new().max_connections(1).connect(url).await?;
	DB.set(pool).expect("db::init called twice");
	Ok(db())
}

pub fn validate<T: FromStr>(msg: &'static str) -> impl FnMut(&str) -> StdResult<(), String> {
	move |s| s.parse::<T>().map(|_| {}).map_err(|_| msg.to_string())
}
