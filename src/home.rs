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
		// .map(|row: PgRow| row.get::<String, _>("data"))
		.fetch_optional(&mut tx)
		.await
		.or_500()?
		.map(|x| x.data);

	if let Some(data) = data {
		tx.commit().await.or_500()?;
		Ok(Html(data))
	} else {
		let articles = query_as!(
			ArticleInfo,
			"SELECT title, date_published as published, date_updated  as updated FROM article
	ORDER BY COALESCE(date_updated, date_published) DESC
	LIMIT 5"
		)
		.fetch_all(&mut tx)
		.await
		.or_500()?;

		let home = Home { articles };

		let home = home.render().or_500()?;
		query!(
			"INSERT INTO home_cache(data)
	VALUES($1)
	ON CONFLICT(_home_id) DO UPDATE
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
