mod files;
mod uri;

use std::{
	borrow::Cow,
	process::Stdio,
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
pub use uri::SftpUri;

fn escape(s: &'_ str) -> Cow<'_, str> {
	shell_escape::unix::escape(Cow::Borrowed(s))
}

pub struct Sftp {
	pub cmd_path: String,
	pub uri: SftpUri,
	pub extra_args: Vec<String>,
	pub ssh_config: Option<String>,
}

impl Sftp {
	fn command(&self) -> Command {
		let mut cmd = Command::new(&self.cmd_path);
		cmd.args(&self.extra_args)
			.args(&["-b", "-"])
			.args(self.uri.port.map(|p| format!("-oPort={p}")))
			.stdin(Stdio::piped())
			.stderr(Stdio::inherit());

		if let Some(path) = &self.ssh_config {
			cmd.arg("-F").arg(path);
		}
		cmd.arg(&self.uri.remote);
		cmd
	}
}

impl Sftp {
	pub async fn send_files(&self, dir: &str, files: &[SendFile]) -> Result<()> {
		assert!(!files.is_empty(), "files passed to `send_files` is empty");
		let dir = escape(dir);

		let mut cmds = vec![
			format!("cd {}", escape(&self.uri.root)),
			format!("-mkdir {dir}"),
			format!("cd {dir}"),
		];
		cmds.extend(files.iter().map(|f| f.sftp_cmd()));

		let mut cmd = self.command();
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
		let cmds = format!(
			"\
			cd {root}
rm {dir}/*
rmdir {dir}\
",
			root = escape(&self.uri.root),
			dir = escape(dir),
		);

		let mut cmd = self.command();
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
