use sqlx::{
	postgres::PgPoolOptions as Opts,
	PgPool as Pool,
};
use tokio::sync::OnceCell;

#[cfg(prod)]
const SCHEMA: &str = include_str!("schema.sql");
#[cfg(not(prod))]
const SCHEMA: &str = "";

static DB: OnceCell<Pool> = OnceCell::const_new();

pub fn db() -> &'static Pool {
	DB.get().expect("DB is not initialized!")
}

pub async fn init(url: &str) -> self::prelude::Result<()> {
	use sqlx::Executor;
	let pool = Opts::new()
		.max_connections(1)
		.max_lifetime(std::time::Duration::from_secs(4 * 3600))
		.idle_timeout(std::time::Duration::from_secs(3600))
		.connect(url)
		.await?;
	{
		let mut tx = pool.begin().await?;
		tx.execute(SCHEMA).await?;
		tx.commit().await?;
	}
	DB.set(pool).expect("db::init called twice");
	Ok(())
}

pub(crate) mod prelude {
	pub use sqlx::{
		postgres::PgRow,
		Execute,
		Row,
	};

	pub use super::db;

	pub type Result<T> = ::std::result::Result<T, sqlx::Error>;

	macro_rules! query {
	($q:expr) => {{
		sqlx::query($q)
	}};
	($q:expr, $($params:expr),* $(,)?) => {{
		sqlx::query($q)
		$(.bind($params))*
	}};
}

	pub(crate) use query;
}
