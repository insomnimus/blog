mod files;
mod uri;

use std::process::Stdio;

use anyhow::{
	ensure,
	Result,
};
pub use files::*;
use shell_words::quote as escape;
use tokio::{
	io::AsyncWriteExt,
	process::Command,
};

pub use self::uri::SftpUri;
use crate::cmd::Cmd;

pub struct Sftp {
	pub cmd: Cmd,
	pub uri: SftpUri,
}

impl Sftp {
	fn command(&self) -> Command {
		let mut cmd = self.cmd.to_tokio();
		cmd
			// .args(&["-b", "-"])
			.args(self.uri.port.map(|p| format!("-oPort={p}")))
			.arg(&self.uri.remote)
			.stdin(Stdio::piped())
			.stderr(Stdio::inherit());

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
			"cd {root}
rm {dir}/*
rmdir {dir}",
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

	pub async fn remove_files<S: AsRef<str>>(&self, files: &[S]) -> Result<()> {
		use std::io::Write;
		assert!(
			!files.is_empty(),
			"files passed to Sftp::remove_files is empty"
		);
		let mut cmds = Vec::new();
		writeln!(cmds, "cd {}", escape(&self.uri.root))?;

		for f in files {
			writeln!(cmds, "rm {}", escape(f.as_ref()))?;
		}

		let mut cmd = self.command();
		let mut proc = cmd.spawn()?;

		let mut stdin = proc.stdin.take().unwrap();

		let handle = tokio::spawn(async move {
			AsyncWriteExt::write_all(&mut stdin, &cmds).await?;
			stdin.shutdown().await
		});

		let status = proc.wait().await?;
		ensure!(status.success(), "sftp command exited with {status}");

		handle.await??;

		Ok(())
	}
}
