use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct Media {
	pub path: String,
	pub alt: Option<String>,
}

impl Media {
	pub fn new<S: Into<String>>(path: S) -> Self {
		Self {
			path: path.into(),
			alt: None,
		}
	}

	pub fn render_html(&self) -> String {
		let path = url_escape::encode_path(&self.path);
		let m = mime_guess::from_path(&self.path).first_or_text_plain();

		match m.type_().as_str() {
			"audio" => {
				format!(
					r#"<audio controls>
<source src="/media/{path}" type="{m}">
Your browser does not support this media.
</audio>"#
				)
			}
			"image" => {
				let alt = self
					.alt
					.as_deref()
					.map(html_escape::encode_text)
					.unwrap_or_default();
				format!(r#"<img src="/media/{path}" alt="{alt}">"#)
			}
			"video" => {
				format!(
					r#"<video controls>
<source src="/media/{path}" type="{m}">
Your browser does not support this media type.
</video>"#
				)
			}
			_ => {
				let name = Path::new(&self.path);
				let name = name
					.file_name()
					.and_then(|name| name.to_str())
					.unwrap_or("unknown");

				let name = html_escape::encode_text(name);

				format!(r#"<a href="/media/{path}" download> Download {name} </a>"#)
			}
		}
	}
}
