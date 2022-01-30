use crate::prelude::*;
use super::Post;

pub fn app() -> App<'static> {
	App::new("delete")
	.about("Delete posts.")
	.group(ArgGroup::new("criteria").args(&["id", "last"]).required(true))
	.args(&[
	arg!(id: [ID] "The ID of the post.")
	.validator(validate::<i32>("the value must be a whole number")),
	arg!(--last "Delete the last post instead."),
	arg!(-y --yes "Do not prompt for confirmation."),
		arg!(-r --sftp [URI] "The sftp servers connection uri in the form `user@domain:/path/to/store`.")
			.env("BLOG_SFTP_URI")
			.validator(validate_sftp_uri),
			Arg::new("sftp-args")
		.multiple_values(true)
		.last(true)
		.help("Extra args to pass to the sftp command.")
		.required(false)
		.requires("sftp"),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let yes = m.is_present("yes");
	
	let mut tx = db().begin().await?;
	
	let post = match m.value_of_t::<i32>("id") {
		Ok(id) => {
			query!(
			r#"SELECT
			p.post_id AS id,
			p.date_posted AS date,
			p.raw,
			ARRAY_AGG(m.file_path) AS "attachments: Vec<Option<String>>"
			FROM post p
			LEFT JOIN post_media m
			ON m.post_id = p.post_id
			WHERE p.post_id = $1
			GROUP BY p.post_id"#,
			id,
			)
			.fetch_optional(&mut tx)
			.await?
			.map(|mut x| Post {
				id: x.id,
				date: x.date.to_local(),
				raw: x.raw.take(),
				rendered: String::new(),
				attachments: x.attachments.take().into_iter().flatten().flatten().collect(),
			})
			.ok_or_else(|| anyhow!("no post found with the id {id}"))?
		}
		// `--last` is set here
		Err(_) => {
			query!(
			r#"SELECT
			p.post_id AS id,
			p.raw,
			p.date_posted AS date,
			ARRAY_AGG(m.file_path) AS "attachments: Vec<Option<String>>"
			FROM post p
			LEFT JOIN post_media m
			ON p.post_id = m.post_id
			GROUP BY p.post_id
			ORDER BY p.post_id DESC
			LIMIT 1"#
			)
			.fetch_optional(&mut tx)
			.await?
						.map(|mut x| Post {
				id: x.id,
				date: x.date.to_local(),
				raw: x.raw.take(),
				rendered: String::new(),
				attachments: x.attachments.take().into_iter().flatten().flatten().collect(),
			})
			.ok_or_else(|| anyhow!("there are no posts in the database"))?
		}
	};
	
	if !post.attachments.is_empty() && !m.is_present("sftp") {
		return Err(anyhow!("the post has {} attachments but no sftp uri was provided", post.attachments.len()));
	}
	
	if !yes {
		println!("post #{}", post.id);
		println!("{}", &post.raw)]);
		let msg = if !post.attachments.is_empty() {
			println!("ATTACHMENTS:");
			for a in &post.attachments {
				println!("-  {a}");
			}
			"Do you want to delete this post and all its attachments?"
		} else {
			"Do you want to delete this post?"
		};
		if !confirm!("{msg}")
		return Ok(());
	}
	
	query!("DELETE FROM post WHERE post_id = $1", post.id)
	.execute(&mut tx)
	.await?;
	
	if !post.attachments.is_empty() {
		let sftp = sftp_args(m);
		let dir = post_dir(post.id);
		sftp.rmdir(&dir).await?;
		println!("✓ deleted attachments from the sftp server");
	}
	
	tx.commit().await?;
	println!("✓ deleted psot #{}", post.id);
	
	Ok(())
}
