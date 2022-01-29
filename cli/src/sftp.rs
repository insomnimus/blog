use std::{
	borrow::Cow,
	path::Path,
	process::Stdio,
	str::FromStr,
};

use anyhow::{
	anyhow,
	ensure,
};
use tokio::{
	io::AsyncWriteExt,
	process::Command,
};

pub enum SftpCommand {
	Provided(String, Vec<String>),
	Constructed {
		cmd_path: String,
		remote: String,
		extra_args: Vec<String>,
	},
}

pub struct Sftp {
	pub cmd: SftpCommand,
	pub root: String,
}

impl SftpCommand {
	fn command(&self) -> Command {
		let mut cmd = match self {
			Self::Provided(path, args) => {
				let mut c = Command::new(path);
				c.args(args);
				c
			}
			Self::Constructed {
				cmd_path,
				remote,
				extra_args,
			} => {
				let mut c = Command::new(cmd_path);
				c.args(extra_args).args(&["-b", "-"]).arg(remote);
				c
			}
		};
		cmd.stdin(Stdio::piped()).stderr(Stdio::inherit());
		cmd
	}
}

impl Sftp {
	pub async fn send_files(&self, dir: &str, files: &[SendFile]) -> anyhow::Result<()> {
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
}

fn escape(s: &'_ str) -> Cow<'_, str> {
	if cfg!(windows) {
		shell_escape::windows::escape(Cow::Borrowed(s))
	} else {
		shell_escape::unix::escape(Cow::Borrowed(s))
	}
}

pub struct SendFile {
	local: String,
	remote: String,
}

impl SendFile {
	pub fn remote(&self) -> &str {
		&self.remote
	}

	fn sftp_cmd(&self) -> String {
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

pub struct SftpUri {
	pub remote: String,
	pub root: String,
}

impl FromStr for SftpUri {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (remote, root) = s.rsplit_once(':').ok_or("missing the `:` separator")?;

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
