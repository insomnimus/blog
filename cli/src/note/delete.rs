use super::Post;
use crate::{
	media,
	prelude::*,
};

pub fn app() -> App {
	App::new("delete")
		.about("Delete notes.")
		.group(ArgGroup::new("handle").args(&["id", "last"]).required(true))
		.args(&[
			arg!(id: [ID] "The ID of the note.")
				.validator(validate::<i32>("the value must be a whole number")),
			arg!(--last "Delete the last note instead."),
			arg!(-y --yes "Do not prompt for confirmation."),
			arg!(--dirty "Do not abort the operation if the attachments could not be deleted."),
			arg!(--"keep-attachments" "Do not attempt to delete attachments.")
				.conflicts_with("dirty"),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let yes = m.is_present("yes");
	let dirty = m.is_present("dirty");
	let keep_attachments = m.is_present("keep-attachments");

	let mut tx = db().begin().await?;

	let note = match m.value_of_t::<i32>("id") {
		Ok(id) => query!(
			r#"SELECT
			n.note_id AS id,
			n.date_posted AS date,
			n.raw,
			ARRAY_AGG(m.file_path) AS "attachments: Vec<Option<String>>"
			FROM note n
			LEFT JOIN note_media m
			ON n.note_id = m.note_id
			WHERE n.note_id = $1
			GROUP BY n.note_id"#,
			id,
		)
		.fetch_optional(&mut tx)
		.await?
		.map(|mut x| Post {
			id: x.id,
			date: x.date.to_local(),
			raw: x.raw.take(),
			rendered: None,
			attachments: x
				.attachments
				.take()
				.into_iter()
				.flatten()
				.flatten()
				.collect(),
		})
		.ok_or_else(|| anyhow!("no post found with the id {id}"))?,
		// `--last` is set here
		Err(_) => query!(
			r#"SELECT
			n.note_id AS id,
			n.raw,
			n.date_posted AS date,
			ARRAY_AGG(m.file_path) AS "attachments: Vec<Option<String>>"
			FROM note n
			LEFT JOIN note_media m
			ON n.note_id = m.note_id
			GROUP BY n.note_id
			ORDER BY n.note_id DESC
			LIMIT 1"#
		)
		.fetch_optional(&mut tx)
		.await?
		.map(|mut x| Post {
			id: x.id,
			date: x.date.to_local(),
			raw: x.raw.take(),
			rendered: None,
			attachments: x
				.attachments
				.take()
				.into_iter()
				.flatten()
				.flatten()
				.collect(),
		})
		.ok_or_else(|| anyhow!("there are no notes in the database"))?,
	};

	let root = if note.attachments.is_empty() || keep_attachments {
		None
	} else {
		Some(Config::media_dir(m).await?)
	};

	if !yes {
		println!("note #{}", note.id);
		println!("{}", &note.raw);
		let msg = if !note.attachments.is_empty() {
			println!("ATTACHMENTS:");
			for a in &note.attachments {
				println!("-  {a}");
			}
			"Do you want to delete this note and all its attachments?"
		} else {
			"Do you want to delete this note?"
		};
		if !confirm!("{msg}")? {
			return Ok(());
		}
	}

	query!("DELETE FROM note WHERE note_id = $1", note.id)
		.execute(&mut tx)
		.await?;

	if let Some(root) = root {
		run_hook!(pre_media, m).await?;
		match media::remove_files(root, &note.attachments).await {
			Ok(_) => {
				println!("✓ deleted attachments from the media directory");
			}
			Err(e) if dirty => {
				eprintln!("warning: failed to delete attachments: {e}");
			}
			Err(e) => return Err(e.into()),
		}
	}

	clear!(notes).execute(&mut tx).await?;
	tx.commit().await?;
	println!("✓ deleted note #{}", note.id);

	Ok(())
}
