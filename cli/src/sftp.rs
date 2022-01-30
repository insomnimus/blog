mod files;

use std::{
	borrow::Cow,
	process::Stdio,
	str::FromStr,
};

use anyhow::{
	ensure,
	Result,
};
pub use files::*;
use tokio::{
	io::AsyncWriteExt,
	process::Command,
};

pub struct SftpUri {
	pub remote: String,
	pub root: String,
}

impl FromStr for SftpUri {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (remote, root) = s.split_once(':').ok_or("missing the `:` separator")?;

		if remote.is_empty() {
			Err("host name is missing")
		} else {
			Ok(Self {
				remote: remote.into(),
				root: root.into(),
			})
		}
	}
}

fn escape(s: &'_ str) -> Cow<'_, str> {
	if cfg!(windows) {
		shell_escape::windows::escape(Cow::Borrowed(s))
	} else {
		shell_escape::unix::escape(Cow::Borrowed(s))
	}
}

pub struct SftpCommand {
	pub cmd_path: String,
	pub remote: String,
	pub extra_args: Vec<String>,
}

pub struct Sftp {
	pub cmd: SftpCommand,
	pub root: String,
}

impl SftpCommand {
	fn command(&self) -> Command {
		let mut cmd = Command::new(&self.cmd_path);
		cmd.args(&self.extra_args)
			.args(&["-b", "-"])
			.arg(&self.remote)
			.stdin(Stdio::piped())
			.stderr(Stdio::inherit());
		cmd
	}
}

impl Sftp {
	pub async fn send_files(&self, dir: &str, files: &[SendFile]) -> Result<()> {
		assert!(!files.is_empty(), "files passed to `send_files` is empty");

		let mut cmds = vec![
			format!("cd {}", &self.root),
			format!("-mkdir {dir}"),
			format!("cd {dir}"),
		];
		cmds.extend(files.iter().map(|f| f.sftp_cmd()));

		let mut cmd = self.cmd.command();
		let mut proc = cmd.spawn()?;

		let mut stdin = proc.stdin.take().unwrap();

		let handle = tokio::spawn(async move {
			for s in cmds {
				// println!("running `{s}`");
				AsyncWriteExt::write_all(&mut stdin, s.as_bytes()).await?;
				AsyncWriteExt::write_all(&mut stdin, b"\n").await?;
			}

			stdin.shutdown().await
		});

		let status = proc.wait().await?;
		ensure!(status.success(), "sftp command exited with {status}");

		handle.await??;

		Ok(())
	}

	pub async fn rmdir(&self, dir: &str) -> Result<()> {
		let cmds = format!("cd {}\nrmdir {}", escape(&self.root), escape(dir),);

		let mut cmd = self.cmd.command();
		let mut proc = cmd.spawn()?;

		let mut stdin = proc.stdin.take().unwrap();

		let handle = tokio::spawn(async move {
			AsyncWriteExt::write_all(&mut stdin, cmds.as_bytes()).await?;
			stdin.shutdown().await
		});

		let status = proc.wait().await?;
		ensure!(status.success(), "sftp command exited with {status}");

		handle.await??;

		Ok(())
	}
}
