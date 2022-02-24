mod files;

use std::path::Path;

use tokio::fs;

pub use self::files::*;

type Result = std::io::Result<()>;

pub async fn send_files<P: AsRef<Path>>(dir: P, files: &[SendFile]) -> Result {
	assert!(!files.is_empty(), "files passed to send_fiels is empty");

	let dir = dir.as_ref();

	for f in files {
		fs::create_dir(dir).await?;
		fs::copy(f.local(), &dir.join(f.remote())).await?;
	}

	Ok(())
}

pub async fn remove_files<P: AsRef<Path>, S: AsRef<str>>(root: P, files: &[S]) -> Result {
	assert!(!files.is_empty(), "files passed to delete_files is empty");

	let root = root.as_ref();

	for f in files {
		fs::remove_file(&root.join(f.as_ref())).await?;
	}

	Ok(())
}
