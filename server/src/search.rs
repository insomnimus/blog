use crate::{
	article::ArticleInfo,
	prelude::*,
};

#[derive(Deserialize)]
pub struct SearchParams {
	query: String,
}

pub async fn handle_search(
	Query(params): Query<SearchParams>,
) -> HttpResponse<Json<Vec<ArticleInfo>>> {
	let q = format!("%{}%", params.query.to_lowercase());

	query!(
		"SELECT title, url_title, date_published AS published, date_updated AS updated FROM article
	WHERE LOWER(title) LIKE $1
	ORDER BY COALESCE(date_updated, date_published) DESC
	LIMIT 25",
		&q,
	)
	.fetch_all(db())
	.await
	.or_500()
	.map(|v| {
		v.into_iter()
			.map(|mut x| ArticleInfo {
				title: x.title.take(),
				url_title: x.url_title.take(),
				published: x.published.format_utc(),
				updated: x.updated.map(|d| d.format_utc()),
			})
			.collect()
	})
	.map(Json)
}
