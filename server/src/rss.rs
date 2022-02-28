use rss::{
	Category,
	ChannelBuilder,
	Guid,
	ItemBuilder,
};

use crate::{
	article,
	music,
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

	let music = music::CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await
		.read()
		.await;
	let articles = article::get_cache().await?.read().await;

	{
		let cached = cache.read().await;
		if cached.time == articles.time && music.time == cached.time && !cached.data.is_empty() {
			return Ok(cached.data.clone());
		}
	}

	let home = crate::SITE_URL.get().unwrap();

	let last_date = articles.time.max(music.time);

	let items = articles
		.data
		.articles
		.values()
		.take(15)
		.map(|a| {
			ItemBuilder::default()
				.title(a.title.clone())
				.link(format!(
					"{home}/articles/{url_title}",
					url_title = &a.url_title
				))
				.description(match a.updated {
					None => a.about.clone(),
					Some(updated) => format!("(updated {}): {}", updated.format_rss(), &a.about),
				})
				.pub_date(a.updated.unwrap_or(a.published).format_rss())
				.guid(Guid {
					value: format!("{} - {}", a.updated.unwrap_or(a.published), a.title),
					permalink: false,
				})
				.build()
		})
		.chain(
			music
				.data
				.music
				.iter()
				.map(|x| {
					ItemBuilder::default()
						.title(x.title.clone())
						.link(format!("{home}/music/{id}", id = x.id))
						.description(x.comment.clone())
						.pub_date(x.date.format_rss())
						.guid(Guid {
							value: format!("{home}/music/{id}", id = x.id),
							permalink: true,
						})
						.build()
				})
				.take(15),
		)
		.collect::<Vec<_>>();

	drop(articles);
	drop(music);

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
