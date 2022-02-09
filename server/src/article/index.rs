use anyhow::Result;
use indexmap::IndexMap;

use crate::prelude::*;

static CACHE: Cache<IndexMap<String, IndexInfo>> = Cache::const_new();

#[derive(Debug, Clone)]
pub struct IndexInfo {
	pub id: i32,
	pub title: String,
	pub url_title: String,
}

pub async fn get_index() -> Result<&'static RwLock<crate::CacheData<IndexMap<String, IndexInfo>>>> {
	let last = query!("SELECT articles FROM cache")
		.fetch_one(db())
		.await?
		.articles;

	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await;

	if cache.read().await.time == last {
		return Ok(cache);
	}

	info!("updating article index");

	let mut stream = query!(
		r#"SELECT
	url_title,
	title,
	article_id AS id,
	date_published
	FROM article
	ORDER BY date_published  ASC"#
	)
	.fetch(db());

	let mut index = IndexMap::new();

	while let Some(res) = stream.next().await {
		let mut x = res?;
		let url_title = x.url_title.clone();
		let key = x.url_title.take();
		index.insert(
			key,
			IndexInfo {
				id: x.id,
				title: x.title.take(),
				url_title,
			},
		);
	}

	let mut cached = cache.write().await;
	cached.data = index;
	cached.time = last;
	drop(cached);

	Ok(cache)
}

pub async fn get_adjacent(
	url_title: &str,
) -> Result<Option<(Option<IndexInfo>, Option<IndexInfo>)>> {
	fn own((_, v): (&String, &IndexInfo)) -> IndexInfo {
		v.clone()
	}

	let cache = get_index().await?.read().await;
	let data = &cache.data;

	Ok(match data.get_index_of(url_title) {
		None => None,
		Some(_) if data.len() == 1 => Some((None, None)),
		Some(0) => {
			Some((
				// There's no previous article.
				None,
				// The next article.
				data.get_index(1).map(own),
			))
		}
		Some(n) if n + 1 == cache.data.len() => {
			Some((
				// Previous article.
				data.get_index(n - 1).map(own),
				// There's no next; this is the last article.
				None,
			))
		}
		Some(n) => {
			Some((
				// Previous
				data.get_index(n - 1).map(own),
				// Next
				data.get_index(n + 1).map(own),
			))
		}
	})
}