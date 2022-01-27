use sqlx::{
	postgres::PgPoolOptions as Opts,
	PgPool as Pool,
};
use tokio::sync::OnceCell;

static DB: OnceCell<Pool> = OnceCell::const_new();

pub fn db() -> &'static Pool {
	DB.get().expect("DB is not initialized!")
}

pub async fn init(url: &str) -> Result<(), sqlx::Error> {
	let pool = Opts::new()
		.max_lifetime(std::time::Duration::from_secs(8 * 3600))
		.idle_timeout(std::time::Duration::from_secs(2 * 3600))
		.connect(url)
		.await?;
	sqlx::query!("TRUNCATE html_cache").execute(&pool).await?;
	DB.set(pool).expect("db::init called twice");
	Ok(())
}
