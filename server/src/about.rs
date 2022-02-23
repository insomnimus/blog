use crate::prelude::*;

#[derive(Template, Debug, Clone)]
#[template(path = "about.html")]
pub struct About {
	html: String,
}

pub async fn handle_about() -> HttpResponse<About> {
	query_as!(About, "SELECT html FROM about")
		.fetch_optional(db())
		.await
		.map_err(|e| e500!(e))?
		.ok_or_else(|| {
			warn!("about page is missing from the database");
			E503
		})
}
