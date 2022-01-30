use std::{
	path::Path,
	str::FromStr,
};

use anyhow::{
	anyhow,
	ensure,
};

use super::escape;

pub struct SendFile {
	local: String,
	remote: String,
}

impl SendFile {
	pub fn remote(&self) -> &str {
		&self.remote
	}

	pub fn sftp_cmd(&self) -> String {
		format!("put {} {}", escape(&self.local), escape(&self.remote))
	}
}

impl FromStr for SendFile {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (local, remote) = match s.split_once("::") {
			Some((left, right)) => {
				ensure!(
					!right.contains(|c: char| if cfg!(windows) {
						c == ':' || c == '/' || c == '\\'
					} else {
						c == '/'
					}),
					"{right}: remote file cannot contain path separators",
				);
				(left, right.to_string())
			}
			None => {
				let p = Path::new(s);
				let fname = p
					.file_name()
					.ok_or_else(|| anyhow!("{s}: failed to determine a remote file name"))?;
				(s, fname.to_string_lossy().into_owned())
			}
		};

		let md = std::fs::metadata(local)
			.map_err(|e| anyhow!("error making sure file exists: {}", e))?;

		if md.file_type().is_file() {
			Ok(Self {
				local: local.into(),
				remote,
			})
		} else {
			Err(anyhow!("file is not a plain file: {}", local))
		}
	}
}
