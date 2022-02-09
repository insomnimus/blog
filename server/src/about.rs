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
		.or_500()?
		.ok_or("about page missing from the database")
		.or_503()
}
