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
	pub date: String,
	pub n_attachments: i64,
}

#[derive(Template)]
#[template(path = "single_post.html")]
pub struct PostPage {
	post: Post,
}

#[derive(Template)]
#[template(path = "posts_page.html")]
pub struct PostsPage {
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

async fn get_posts(last_id: i32) -> DbResult<Vec<Post>> {
	query!(
		r#"SELECT
	p.post_id AS id,
	p.content,
	p.date_posted AS date,
	ARRAY_AGG(m.file_path) AS "attachments: Vec<Option<String>>"
	FROM post p
	LEFT JOIN post_media m
	ON p.post_id = m.post_id
	WHERE $1 = 1 OR p.post_id < $1
	GROUP BY p.post_id
	ORDER BY p.date_posted DESC
	LIMIT 25"#,
		last_id,
	)
	.fetch(db())
	.map_ok(|mut x| Post {
		id: x.id,
		content: x.content.take(),
		date: x.date.format_utc(),
		attachments: x
			.attachments
			.take()
			.into_iter()
			.flatten()
			.flatten()
			.map(Media::new)
			.collect(),
	})
	.try_collect()
	.await
}

pub async fn handle_posts() -> HttpResponse {
	static CACHE: Cache = Cache::const_new();

	async fn inner() -> Result<Html<String>> {
		let cache = CACHE
			.get_or_init(|| async { RwLock::new(Default::default()) })
			.await;

		let last_updated = query!("SELECT posts FROM cache")
			.fetch_one(db())
			.await?
			.posts;

		{
			let cached = cache.read().await;
			if cached.time == last_updated && !cached.data.is_empty() {
				return Ok(Html(cached.data.clone()));
			}
		}
		debug!("updating posts cache");

		let posts = get_posts(1).await?;
		let html = PostsPage { posts }.render()?;

		let mut cached = cache.write().await;
		cached.data.clear();
		cached.data.push_str(&html);
		cached.time = last_updated;

		Ok(Html(html))
	}

	match inner().await {
		Ok(x) => Ok(x),
		Err(e) => {
			error!("{e}");
			Err(E500)
		}
	}
}

pub async fn handle_api(Query(params): Query<PostParams>) -> HttpResponse<Json<PostsJson>> {
	get_posts(params.cursor)
		.await
		.map_err(|e| {
			error!(target: "api/posts", "{e}");
			E500
		})?
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
	.map_err(|e| {
		error!("{e}");
		E500
	})?
	.or_404()
	.map(|mut x| PostPage {
		post: Post {
			id: x.id,
			content: x.content.take(),
			date: x.date.format_utc(),
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
