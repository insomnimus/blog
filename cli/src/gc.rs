use std::collections::{
	HashMap,
	HashSet,
};

use tokio::fs;

use crate::prelude::*;

pub fn app() -> App {
	App::new("gc")
		.about("Run cleanup on the database and the media directory.")
		.args(&[
			arg!(--dry "Do not perform any cleanup; only display actions."),
			arg!(--"only-db" "Do not do cleanup on the media directory; only clean the database."),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	gc_db(m).await?;
	if !m.is_present("only-db") {
		gc_media(m).await?;
	}

	Ok(())
}

async fn gc_db(m: &ArgMatches) -> Result<()> {
	let dry = m.is_present("dry");
	let mut tx = db().begin().await?;
	let tags = query!(
		"SELECT t.tag_name
FROM tag t
WHERE NOT EXISTS (
	SELECT FROM article_tag art WHERE art.tag_name = t.tag_name
)"
	)
	.fetch(&mut tx)
	.map_ok(|x| x.tag_name)
	.try_collect::<Vec<_>>()
	.await?;

	if !tags.is_empty() {
		println!(
			"deleting {} unreferenced tags from the database",
			tags.len()
		);
		if !dry {
			query!("DELETE FROM tag WHERE tag_name = ANY($1)", &tags)
				.execute(&mut tx)
				.await?;
		}
	}

	let media = query!(
		"SELECT m.file_path
FROM media m
WHERE NOT EXISTS (
	SELECT other.file_path FROM(
		SELECT file_path FROM music mu
		UNION
		SELECT file_path FROM post_media p
	) other
	WHERE m.file_path = other.file_path
)"
	)
	.fetch(&mut tx)
	.map_ok(|x| x.file_path)
	.try_collect::<Vec<_>>()
	.await?;

	if !media.is_empty() {
		println!(
			"deleting {} unreferenced media entries from the database",
			media.len()
		);
		if !dry {
			query!("DELETE FROM media WHERE file_path = ANY($1)", &media)
				.execute(&mut tx)
				.await?;
		}
	}

	if !dry {
		tx.commit().await?;
	}

	Ok(())
}

async fn gc_media(m: &ArgMatches) -> Result<()> {
	let dry = m.is_present("dry");
	let root = Config::media_dir(m).await?;
	run_hook!(pre_media, m).await?;
	let mut dirs = task::block_in_place(|| {
		let mut dirs = HashMap::new();
		for res in root.read_dir()? {
			let entry = res?;
			if entry.file_type()?.is_dir() {
				let mut files = Vec::new();
				for file in entry.path().read_dir()? {
					files.push(file?.file_name());
				}
				dirs.insert(entry.file_name(), files);
			}
		}
		Ok::<_, std::io::Error>(dirs)
	})?;

	let media_files = query!("SELECT file_path FROM media")
		.fetch(db())
		.map_ok(|x| x.file_path)
		.try_collect::<HashSet<_>>()
		.await?;

	let mut to_delete = Vec::new();
	for (dir, files) in &mut dirs {
		let dir = match dir.to_str() {
			None => continue,
			Some(s) => s,
		};
		files.retain(|file| {
			let file = match file.to_str() {
				None => return true,
				Some(s) => s,
			};
			// String formatting is intentional; the values in the database are in the form
			// dir/file.
			let path = format!("{dir}/{file}");
			if media_files.contains(&path) {
				true
			} else {
				to_delete.push(path);
				false
			}
		});
	}

	for f in &to_delete {
		println!("deleting unreferenced file: {f}");
		if !dry {
			fs::remove_file(&root.join(f)).await?;
		}
	}

	for (dir, _) in dirs.iter().filter(|(_, files)| files.is_empty()) {
		println!("deleting empty directory: {dir:?}");
		if !dry {
			fs::remove_dir(&root.join(dir)).await?;
		}
	}

	Ok(())
}
