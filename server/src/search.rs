#[derive(Serialize)]
struct SearchResult {
	title: String,
	url_title: String,
	published: String,
	updated: Option<String>,
}

pub async fn handle_search(Path(query): Path<String>) -> HttpResponse<Json> {
	
}