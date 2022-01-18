use crate::{
	article::ArticleInfo,
	prelude::*,
};

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
	articles: Vec<ArticleInfo>,
}

pub async fn handle_home() -> HttpResponse {
	let mut tx = db().begin().await.or_500()?;
	let data = query!("SELECT data FROM home_cache")
		.fetch_optional(&mut tx)
		.await
		.or_500()?
		.map(|x| x.data);

	if let Some(data) = data {
		tx.commit().await.or_500()?;
		Ok(Html(data))
	} else {
		// NOTE: Do not use query_as here, it panics for some reason.
		let articles = query!(
			r#"SELECT title,
			url_title,
			date_published AS published,
			date_updated AS updated
			FROM article
	ORDER BY COALESCE(date_updated, date_published) DESC
	LIMIT 5"#
		)
		.fetch_all(&mut tx)
		.await
		.or_500()?
		.into_iter()
		.map(|mut x| ArticleInfo {
			title: x.title.take(),
			url_title: x.url_title.take(),
			published: x.published.format_utc(),
			updated: x.updated.map(|d| d.format_utc()),
		})
		.collect::<Vec<_>>();

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
