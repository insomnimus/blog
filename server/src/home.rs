use crate::{
	article::ArticleInfo,
	post::PostInfo,
	prelude::*,
};

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
	articles: Vec<ArticleInfo>,
	posts: Vec<PostInfo>,
}

pub async fn handle_home() -> HttpResponse {
	let mut tx = db().begin().await.or_500()?;
	let data = query!("SELECT home_page FROM html_cache")
		.fetch_optional(&mut tx)
		.await
		.or_500()?
		.and_then(|x| x.home_page);

	if let Some(data) = data {
		tx.commit().await.or_500()?;
		return Ok(Html(data));
	}

	let articles = query!(
		"SELECT title,
			url_title,
			about,
			date_published AS published,
			date_updated AS updated
			FROM article
	ORDER BY COALESCE(date_updated, date_published) DESC
	LIMIT 5"
	)
	.fetch_all(&mut tx)
	.await
	.or_500()?
	.into_iter()
	.map(|mut x| ArticleInfo {
		title: x.title.take(),
		url_title: x.url_title.take(),
		about: x.about.take(),
		published: x.published.format_utc(),
		updated: x.updated.map(|d| d.format_utc()),
		tags: Vec::new(),
	})
	.collect::<Vec<_>>();

	let posts = query_as!(
		PostInfo,
		r#"SELECT p.post_id AS id,
		p.content,
		p.date_posted AS date,
		COALESCE(COUNT(m.file_path), 0) AS "n_attachments!"
	FROM post p
	LEFT JOIN post_media m
	ON m.post_id = p.post_id
	GROUP BY p.post_id
	LIMIT 10"#
	)
	.fetch_all(&mut tx)
	.await
	.or_500()?;

	let home = Home { articles, posts };

	let home = home.render().or_500()?;
	query!(
		"INSERT INTO html_cache(_instance, home_page)
		VALUES('TRUE', $1)
		ON CONFLICT(_instance) DO UPDATE
		SET home_page = $1",
		home.as_str(),
	)
	.execute(&mut tx)
	.await
	.or_500()?;
	tx.commit().await.or_500()?;
	Ok(Html(home))
}
