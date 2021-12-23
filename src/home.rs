use indexmap::IndexMap;

use crate::prelude::*;

#[derive(Template)]
#[template(path = "home.html")]
pub struct Home {
	pub posts: IndexMap<String, Post>,
}

impl Home {
	fn recent(&'_ self, n: usize) -> impl IntoIterator<Item = &'_ Post> {
		self.posts.values().take(n)
	}
}
