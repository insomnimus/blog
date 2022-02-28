use askama::Result;
use sqlx::types::chrono::NaiveDateTime;

use crate::ext::*;

pub fn date(d: &NaiveDateTime) -> Result<String> {
	let s = format!(r#"<time datetime="{t}">{t}</time>"#, t = d.format_utc(),);

	Ok(s)
}
