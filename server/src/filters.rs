use askama::Result;

use crate::ext::*;

pub fn date<D>(d: &D) -> Result<String>
where
	D: DateTimeExt<Output = String>,
{
	let s = format!(r#"<time datetime="{t}">{t}</time>"#, t = d.format_utc(),);

	Ok(s)
}
