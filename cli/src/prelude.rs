pub use anyhow::Result;
pub use clap::{
	arg,
	App,
	AppSettings,
	Arg,
	ArgMatches,
};
use sqlx::{
	postgres::PgPoolOptions as Opts,
	PgPool as Pool,
};
pub use sqlx::{
	query,
	types::chrono::{
		DateTime,
		Utc,
	},
};
use tokio::{
	runtime::Runtime,
	sync::OnceCell,
};

pub static RT: OnceCell<Runtime> = OnceCell::const_new();
static DB: OnceCell<Pool> = OnceCell::const_new();

macro_rules! block {
	($code:expr) => {{
		let rt = match $crate::prelude::RT.get() {
			None => {
				$crate::prelude::RT
					.set(
						tokio::runtime::Builder::new_current_thread()
							.enable_all()
							.build()
							.unwrap_or_else(|e| {
								eprintln!("error: failed to initialize the async runtime: {}", e);
								std::process::exit(1);
							}),
					)
					.unwrap();
				$crate::prelude::RT.get().unwrap()
			}
			Some(rt) => rt,
		};
		rt.block_on($code)
	}};
}

pub(crate) use block;

pub fn encode_url_title(s: &str) -> String {
	let s = s
		.replace(|c: char| c.is_whitespace() || c == '_', "-")
		.to_lowercase();
	url_escape::encode_component(&s).to_string()
}

pub fn db() -> &'static Pool {
	DB.get().expect("DB is not initialized!")
}

pub async fn init_db(url: &str) -> Result<&'static Pool> {
	let pool = Opts::new().max_connections(1).connect(url).await?;
	DB.set(pool).expect("db::init called twice");
	Ok(db())
}
