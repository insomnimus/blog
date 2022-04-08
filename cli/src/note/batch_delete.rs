use std::collections::HashSet;

use crate::{
	media,
	prelude::*,
};

pub fn app() -> App {
	App::new("batch-delete")
		.about("Batch delete notes.")
		.args(&[
			arg!(id: <ID> "The ID of the note.")
				.multiple_values(true)
				.validator(validate::<i32>("the value must be a whole number")),
			arg!(--"keep-attachments" "Do not attempt to delete attachments."),
			arg!(-v --verify "Verify that all IDs are in the database and fail if not."),
		])
}

fn print_deleted(n: usize, word: &str) {
	let s = if n == 1 { "" } else { "s" };
	println!("✓ deleted {n} {word}{s}");
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let keep_attachments = m.is_present("keep-attachments");
	let ids = m.values_of_t_or_exit::<i32>("id");

	if m.is_present("verify") {
		let db_ids = query!("SELECT note_id FROM note WHERE note_id = ANY($1)", &ids)
			.fetch(db())
			.map_ok(|x| x.note_id)
			.try_collect::<HashSet<_>>()
			.await?;

		let missing = ids
			.iter()
			.filter(|n| !db_ids.contains(n))
			.collect::<Vec<_>>();
		ensure!(
			missing.is_empty(),
			"not all IDs past exist in the database: {missing:?}"
		);
	}

	if keep_attachments {
		let n_deleted = query!("DELETE FROM note WHERE note_id = ANY($1)", &ids)
			.execute(db())
			.await?
			.rows_affected();
		print_deleted(n_deleted as _, "note");
		return Ok(());
	}

	let mut tx = db().begin().await?;

	let media = query!(
		"SELECT file_path FROM note_media WHERE note_id = ANY($1)",
		&ids
	)
	.fetch(&mut tx)
	.map_ok(|x| x.file_path)
	.try_collect::<Vec<_>>()
	.await?;

	if media.is_empty() {
		let n_deleted = query!("DELETE FROM note WHERE note_id = ANY($1)", &ids)
			.execute(&mut tx)
			.await?
			.rows_affected();
		if n_deleted > 0 {
			clear!(notes).execute(&mut tx).await?;
			tx.commit().await?;
		}
		print_deleted(n_deleted as _, "note");
		return Ok(());
	}

	let root = Config::media_dir()?;
	let n_deleted = query!("DELETE FROM note WHERE note_id = ANY($1)", &ids)
		.execute(&mut tx)
		.await?
		.rows_affected();
	query!("DELETE FROM media WHERE file_path = ANY($1)", &media)
		.execute(&mut tx)
		.await?;
	clear!(notes).execute(&mut tx).await?;
	tx.commit().await?;

	print_deleted(n_deleted as _, "note");

	run_hook!(pre_media).await?;
	media::remove_files(root, &media).await?;
	print_deleted(media.len(), "media file");

	Ok(())
}
