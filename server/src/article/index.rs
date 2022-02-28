use indexmap::IndexMap;

use super::ArticleInfo;
use crate::prelude::*;

static CACHE: Cache<IndexMap<String, ArticleInfo>> = Cache::const_new();

pub async fn get_index(
) -> DbResult<&'static RwLock<crate::CacheData<IndexMap<String, ArticleInfo>>>> {
	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await;

	let last = query!("SELECT articles FROM cache")
		.fetch_one(db())
		.await?
		.articles;

	if cache.read().await.time == last {
		return Ok(cache);
	}
	debug!("updating article index");

	let index = query!(
		r#"SELECT
	url_title,
	title,
	about,
	-- article_id AS id,
	date_published AS published,
	date_updated AS updated
	FROM article
	ORDER BY published ASC"#
	)
	.fetch(db())
	.map_ok(|mut x| {
		let url_title = x.url_title.clone();
		let key = x.url_title.take();
		(
			key,
			ArticleInfo {
				title: x.title.take(),
				url_title,
				about: x.about.take(),
				published: x.published,
				updated: x.updated,
				tags: Vec::new(),
			},
		)
	})
	.try_collect::<IndexMap<_, _>>()
	.await?;

	let mut cached = cache.write().await;
	cached.data = index;
	cached.time = last;
	drop(cached);

	Ok(cache)
}

pub async fn get_adjacent(
	url_title: &str,
) -> DbResult<Option<(Option<ArticleInfo>, Option<ArticleInfo>)>> {
	fn own((_, v): (&String, &ArticleInfo)) -> ArticleInfo {
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
