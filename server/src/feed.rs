use atom_syndication::{
	CategoryBuilder,
	EntryBuilder,
	FeedBuilder,
	LinkBuilder,
};

use crate::{
	article,
	music,
	prelude::*,
};

pub async fn handle_feed() -> HttpResponse<Xml<String>> {
	gen_feed().await.map(Xml).map_err(|e| e500!(e))
}

async fn gen_feed() -> anyhow::Result<String> {
	static CACHE: Cache<String> = Cache::const_new();

	let cache = CACHE
		.get_or_init(|| async { RwLock::new(Default::default()) })
		.await;

	let music = music::get_cache().await?.read().await;
	let articles = article::get_cache().await?.read().await;

	let last = articles.time.max(music.time);

	{
		let cached = cache.read().await;
		if cached.time == articles.time && music.time == cached.time && !cached.data.is_empty() {
			return Ok(cached.data.clone());
		}
	}
	let home = crate::SITE_URL.get().unwrap();

	let entries = articles
		.data
		.articles
		.values()
		.take(15)
		.map(|a| {
			EntryBuilder::default()
				.title(a.title.as_str())
				.updated(a.updated.unwrap_or(a.published).to_utc())
				.published(Some(a.published.to_utc().into()))
				.id(format!(
					"{home}/articles/{url_title}",
					url_title = &a.url_title
				))
				.summary(Some(a.about.as_str().into()))
				.build()
		})
		.chain(music.data.music.iter().take(15).map(|x| {
			EntryBuilder::default()
			.title(x.title.clone().unwrap_or_else(|| String::from("Untitled")))
			.summary(x.comment.as_deref().map(|s| s.into()))
			.id(format!("{home}/music/{id}", id = x.id))
			// .link(Some(format!("{home}/music/{id}", id = x.id)))
		.published(Some(x.date.to_utc().into()))
		.updated(x.date.to_utc())
		.build()
		}))
		.collect::<Vec<_>>();

	let feed = FeedBuilder::default()
		.title("Strange Aeons")
		.id(home)
		.updated(last.to_utc())
		.categories([
			CategoryBuilder::default()
				.term("programming")
				.label(Some("Programming".into()))
				.build(),
			CategoryBuilder::default()
				.term("music")
				.label(Some("Music".into()))
				.build(),
		])
		.lang(Some("en".into()))
		.base(home.clone())
		.entries(entries)
		.links([
			LinkBuilder::default()
				.title(Some("Articles".into()))
				.href(format!("{home}/articles"))
				.hreflang(Some("en".into()))
				.rel("self")
				.build(),
			LinkBuilder::default()
				.title(Some("Music".into()))
				.href(format!("{home}/music"))
				.hreflang(Some("en".into()))
				.rel("self")
				.build(),
		])
		.build()
		.to_string();

	drop(articles);
	drop(music);

	let mut cached = cache.write().await;
	cached.data.clear();
	cached.data.push_str(&feed);
	cached.time = last;
	Ok(feed)
}
