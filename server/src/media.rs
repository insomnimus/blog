use std::path::Path;

#[derive(Clone, Debug)]
pub struct Media {
	pub path: String,
	pub kind: MediaType,
}

impl Default for Media {
	fn default() -> Self {
		Self {
			kind: MediaType::Other,
			path: String::new(),
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MediaType {
	Audio,
	Image,
	Video,
	Other,
}

impl Media {
	pub fn new<S: Into<String>>(path: S) -> Self {
		// TODO: Rework this function.
		fn ends(exts: &[&str], s: &str) -> bool {
			exts.iter().any(|ext| s.ends_with(ext))
		}

		let path = path.into();

		let kind = if ends(&[".ogg", ".mp3", ".wav"], &path) {
			MediaType::Audio
		} else if ends(&[".mp4", ".webm"], &path) {
			MediaType::Video
		} else if ends(&[".gif", ".png", ".jpeg", ".jpg"], &path) {
			MediaType::Image
		} else {
			MediaType::Other
		};
		Self { path, kind }
	}

	pub fn render_html(&self) -> String {
		let path = url_escape::encode_path(&self.path);

		match self.kind {
			MediaType::Audio => {
				let ext = if self.path.ends_with(".ogg") {
					"ogg"
				} else if self.path.ends_with(".mp3") {
					"mpeg"
				} else {
					"wav"
				};

				format!(
					r#"<audio controls>
<source src="/media/{path}" type="audio/{ext}">
Your browser does not support this media.
</audio>"#
				)
			}
			MediaType::Image => {
				format!(r#"<img src="/media/{path}">"#)
			}
			MediaType::Video => {
				let ext = if self.path.ends_with(".mp4") {
					"mp4"
				} else if self.path.ends_with(".ogg") {
					"ogg"
				} else {
					"webm"
				};
				format!(
					r#"<video controls>
<source src="/media/{path}" type="video/{ext}">
Your browser does not support this media type.
</video>"#
				)
			}
			MediaType::Other => {
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
