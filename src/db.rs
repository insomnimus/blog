use sqlx::{
	postgres::PgPoolOptions as Opts,
	PgPool as Pool,
};
use tokio::sync::OnceCell;

static DB: OnceCell<Pool> = OnceCell::const_new();

pub fn db() -> &'static Pool {
	DB.get().expect("DB is not initialized!")
}

pub async fn init(url: &str) -> self::prelude::Result<()> {
	let pool = Opts::new()
		.max_lifetime(std::time::Duration::from_secs(8 * 3600))
		.idle_timeout(std::time::Duration::from_secs(2 * 3600))
		.connect(url)
		.await?;
	DB.set(pool).expect("db::init called twice");
	Ok(())
}

pub(crate) mod prelude {
	pub use sqlx::{
		postgres::PgRow,
		query,
		query_as,
		Execute,
		Row,
	};

	pub use super::db;

	pub type Result<T> = ::std::result::Result<T, sqlx::Error>;

	macro_rules! _query_c {
	($q:expr) => {{
		sqlx::query($q)
	}};
	($q:expr, $($params:expr),* $(,)?) => {{
		sqlx::query($q)
		$(.bind($params))*
	}};
}

	// pub(crate) use query_c;
}
