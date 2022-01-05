use crate::{
	article::ArticleInfo,
	prelude::*,
};

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
	articles: Vec<ArticleInfo>,
}

pub async fn handle_home() -> HtmlResponse {
	let mut tx = db().begin().await.or_500()?;
	let data = query!("SELECT data FROM home_cache")
		.map(|row: PgRow| row.get::<String, _>("data"))
		.fetch_optional(&mut tx)
		.await
		.or_500()?;

	if let Some(data) = data {
		tx.commit().await.or_500()?;
		Ok(Html(data))
	} else {
		let articles = query!(
			"SELECT title, date_published, date_updated FROM article
	SORT BY COALESCE(date_updated, date_published) DESC
	LIMIT 5",
		)
		.map(|row: PgRow| ArticleInfo {
			title: row.get("title"),
			published: row.get("date_published"),
			updated: row.get("date_updated"),
		})
		.fetch_all(&mut tx)
		.await
		.or_500()?;

		let home = Home { articles };

		let home = home.render().or_500()?;
		query!(
			"INSERT INTO home_cache(data)
	VALUES($1)
	ON CONFLICT DO UPDATE
	SET data = $1",
			home.as_str(),
		)
		.execute(&mut tx)
		.await
		.or_500()?;
		tx.commit().await.or_500()?;
		Ok(Html(home))
	}
}
