use rss::{
	Category,
	ChannelBuilder,
	Guid,
	ItemBuilder,
};

use crate::{
	article::index,
	prelude::*,
};

pub async fn handle_rss() -> HttpResponse<Xml<String>> {
	gen_feed().await.map(Xml).map_err(|e| e500!(e))
}

async fn gen_feed() -> anyhow::Result<String> {
	static CACHE: Cache<String> = Cache::const_new();
	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await;
	let articles = index::get_index().await?.read().await;
	{
		let cached = cache.read().await;
		if cached.time == articles.time && !cached.data.is_empty() {
			return Ok(cached.data.clone());
		}
	}

	let home = crate::SITE_URL.get().unwrap();

	let last_date = articles.time;

	let items = articles
		.data
		.clone()
		.sorted_by(|_, a, _, b| {
			b.updated
				.unwrap_or(b.published)
				.cmp(&a.updated.unwrap_or(a.published))
		})
		.take(15)
		.map(|(_, mut a)| {
			ItemBuilder::default()
				.title(a.title.take())
				.link(format!(
					"{home}/articles/{url_title}",
					url_title = &a.url_title
				))
				.description(match a.updated {
					None => a.about.take(),
					Some(_) => format!("(update): {}", &a.about),
				})
				.pub_date(a.updated.unwrap_or(a.published).format_rss())
				.guid(Guid {
					value: a.title.take(),
					permalink: false,
				})
				.build()
		})
		.collect::<Vec<_>>();

	drop(articles);

	let ch = ChannelBuilder::default()
		.title("Articles")
		.link(format!("{home}/articles"))
		.language("en".to_string())
		.last_build_date(last_date.format_rss())
		.category(Category {
			name: "blog".into(),
			domain: None,
		})
		.description("Published articles.".to_string())
		.items(items)
		.build();

	let feed = ch.to_string();
	let mut cached = cache.write().await;
	cached.data.clear();
	cached.data.push_str(&feed);
	cached.time = last_date;
	Ok(feed)
}
