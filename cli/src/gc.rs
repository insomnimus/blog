use crate::prelude::*;

pub fn app() -> App {
	App::new("gc")
	.about("Run cleanup on the database and the sftp server.")
			.args(&[
					arg!(-R --sftp [URL] "The sftp servers connection url in the form `sftp://[user@]domain[:port]/path/to/store`.")
			.env("BLOG_SFTP_URL"),
			arg!(--"sftp-command" [COMMAND] "The sftp command. By default it is `sftp -b -`")
			.validator(validate::<crate::cmd::Cmd>("the sftp command is not valid")),
			arg!(--dry "Do not perform any cleanup; only display actions."),
			arg!(--"only-db" "Do not do cleanup on the sftp server; only clean the database."),
		])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let db = Config::database(m).await?;
	run_hook!(pre_db, m).await?;
	init_db(db).await?;

	gc_db(m).await?;
	if !m.is_present("only-db") {
		gc_sftp(m).await?;
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

async fn gc_sftp(m: &ArgMatches) -> Result<()> {
	// let dry = m.is_present("dry");
	unimplemented!()
}
