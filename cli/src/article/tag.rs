pub async fn run(m: &ArgMatches) -> Result<()> {
		// Clear current tags.
		let removed = query!("DELETE FROM article_tag WHERE article_id = $1 RETURNING tag_name", id)
		.fetch_all(&mut tx)
		.await?;
		for t in removed {
			println!("✓ untagged {}", t.tag_name);
		}
		// Push new tags.
		for tag in tags {
				let affected = query!(
				"INSERT INTO tag(tag_name) VALUES($1) ON CONFLICT(tag_name) DO NOTHING",
				tag
			)
			.execute(&mut tx)
			.await?
			.rows_affected();
			if affected > 0 {
				println!("✓ Created new tag '{}'", tag);
			}
			
					query!(
				"INSERT INTO article_tag(article_id, tag_name) VALUES($1, $2)",
				id,
				tag
			)
			.execute(&mut tx)
			.await?;
			println!("✓ tagged {}", tag);
		}
}