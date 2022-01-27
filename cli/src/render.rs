use std::borrow::Cow;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Syntax {
	Bare,
	Markdown,
	Html,
}

impl std::str::FromStr for Syntax {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.eq_ignore_ascii_case("bare") {
			Ok(Self::Bare)
		} else if s.eq_ignore_ascii_case("markdown") {
			Ok(Self::Markdown)
		} else if s.eq_ignore_ascii_case("html") {
			Ok(Self::Html)
		} else {
			Err("value must be one of [bare, markdown, html]")
		}
	}
}

impl Syntax {
	pub const VALUES: &'static [&'static str] = &["bare", "markdown", "html"];

	pub fn render(self, s: &'_ str) -> Cow<'_, str> {
		match self {
			Self::Bare => html_escape::encode_text(s),
			Self::Html => Cow::Borrowed(s),
			Self::Markdown => {
				use comrak::{
					markdown_to_html,
					ComrakExtensionOptions,
					ComrakOptions,
					ComrakParseOptions,
					ComrakRenderOptions,
				};
				let opts = ComrakOptions {
					extension: ComrakExtensionOptions {
						strikethrough: true,
						tagfilter: true,
						table: true,
						autolink: true,
						tasklist: true,
						superscript: true,
						header_ids: None,
						footnotes: true,
						description_lists: true,
						front_matter_delimiter: Some("---".into()),
					},
					parse: ComrakParseOptions {
						smart: true,
						default_info_string: None,
					},
					render: ComrakRenderOptions {
						hardbreaks: false,
						github_pre_lang: true,
						unsafe_: true,
						escape: false,
						..Default::default()
					},
				};

				markdown_to_html(s, &opts).into()
			}
		}
	}
}
