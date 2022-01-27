use crate::{
	media::Media,
	prelude::*,
};

#[derive(Template)]
#[template(path = "post.html")]
pub struct Post {
	id: i32,
	content: String,
	date: String,
	attachments: Vec<Media>,
}

pub struct PostInfo {
	pub id: i32,
	pub content: String,
	pub n_attachments: i64,
	pub date: NaiveDateTime,
}

#[derive(Template)]
#[template(path = "single_post.html")]
pub struct PostPage {
	post: Post,
}

#[derive(Template)]
#[template(path = "posts.html")]
pub struct Posts {
	posts: Vec<Post>,
}

#[derive(Serialize)]
pub struct PostsJson {
	posts: Vec<String>,
}

#[derive(Deserialize)]
pub struct PostParams {
	cursor: i32,
}

async fn get_posts(last_id: i32) -> anyhow::Result<Vec<Post>> {
	query!(
		"SELECT
	p.post_id AS id,
	p.content,
	p.date_posted AS date,
	ARRAY_AGG(m.file_path) AS attachments
	FROM post p
	LEFT JOIN post_media m
	ON p.post_id = m.post_id
	WHERE $1 = 0 OR p.post_id < $1
	GROUP BY p.post_id
	ORDER BY p.post_id DESC
	LIMIT 25",
		last_id.saturating_sub(1),
	)
	.fetch_all(db())
	.await
	.map(|v| {
		v.into_iter()
			.map(|mut x| Post {
				id: x.id,
				content: x.content.take(),
				date: x.date.format_utc(),
				attachments: x
					.attachments
					.take()
					.into_iter()
					.flatten()
					.map(Media::new)
					.collect(),
			})
			.collect()
	})
	.map_err(|e| e.into())
}

pub async fn handle_posts() -> HttpResponse<Posts> {
	get_posts(0).await.or_500().map(|posts| Posts { posts })
}

pub async fn handle_api(Query(params): Query<PostParams>) -> HttpResponse<Json<PostsJson>> {
	get_posts(params.cursor)
		.await
		.or_500()?
		.into_iter()
		.map(|p| p.render())
		.collect::<Result<Vec<_>, _>>()
		.or_500()
		.map(|posts| Json(PostsJson { posts }))
}

pub async fn handle_post(Path(id): Path<i32>) -> HttpResponse<PostPage> {
	query!(
		r#"SELECT
	p.post_id AS id,
	p.content,
	p.date_posted AS date,
	ARRAY_AGG(m.file_path) AS "attachments?: Vec<Option<String>>"
	FROM post p
	LEFT JOIN post_media m
	ON p.post_id = m.post_id
	WHERE p.post_id = $1
	GROUP BY p.post_id"#,
		id
	)
	.fetch_optional(db())
	.await
	.or_500()?
	.or_404()
	.map(|mut x| PostPage {
		post: Post {
			id: x.id,
			content: x.content.take(),
			date: x.date.format_utc(),
			// attachments: vec![],
			attachments: x
				.attachments
				.take()
				.into_iter()
				.flatten()
				.flatten()
				.map(Media::new)
				.collect(),
		},
	})
}
