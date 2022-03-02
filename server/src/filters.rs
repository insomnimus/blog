use std::borrow::Cow;

use askama::Result;
use sqlx::types::chrono::NaiveDateTime;

use crate::ext::*;

pub fn date(d: &NaiveDateTime) -> Result<String> {
	let s = format!(r#"<time datetime="{t}">{t}</time>"#, t = d.format_utc(),);

	Ok(s)
}

pub fn first_sentence<S>(s: &'_ S, len: usize) -> Result<Cow<'_, str>>
where
	S: SplitWords,
{
	Ok(s.first_line_words(len))
}
