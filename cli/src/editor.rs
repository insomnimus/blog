use std::{
	borrow::Cow,
	env,
	fs,
	io::Write,
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use anyhow::{
	anyhow,
	ensure,
	Result,
};
pub use tempfile::Builder;

use crate::cmd::Cmd;

static ENV_VARS: &[&str] = &["VISUAL", "EDITOR"];

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
static HARDCODED_NAMES: &[&str] = &[
	// CLI editors
	"nano",
	"pico",
	"vim",
	"nvim",
	"vi",
	"emacs",
	// GUI editors
	"code",
	"atom",
	"subl",
	"gedit",
	"gvim",
	// Generic "file openers"
	"xdg-open",
	"gnome-open",
	"kde-open",
];

#[cfg(target_os = "macos")]
static HARDCODED_NAMES: &[&str] = &[
	// CLI editors
	"nano",
	"pico",
	"vim",
	"nvim",
	"vi",
	"emacs",
	// open has a special flag to open in the default text editor
	// (this really should come before the CLI editors, but in order
	// not to break compatibility, we still prefer CLI over GUI)
	"open -Wt",
	// GUI editors
	"code -w",
	"atom -w",
	"subl -w",
	"gvim",
	"mate",
	// Generic "file openers"
	"open -a TextEdit",
	"open -a TextMate",
	// TODO: "open -f" reads input from standard input and opens with
	// TextEdit. if this flag were used we could skip the tempfile
	"open",
];

#[cfg(target_os = "windows")]
static HARDCODED_NAMES: &[&str] = &[
	// GUI editors
	"code.cmd -n -w",
	"atom.exe -w",
	"subl.exe -w",
	// notepad++ does not block for input
	// Installed by default
	"notepad.exe",
	// Generic "file openers"
	"cmd.exe /C start",
];

fn editor_cmd() -> Option<(PathBuf, Vec<String>)> {
	ENV_VARS
		.iter()
		.flat_map(env::var)
		.filter(|s| !s.is_empty())
		.map(Cow::Owned)
		.chain(HARDCODED_NAMES.iter().cloned().map(Cow::Borrowed))
		.filter_map(|s| {
			let cmd = s.parse::<Cmd>().ok()?;
			let path = which::which(&cmd.cmd).ok()?;
			Some((path, cmd.args))
		})
		.next()
}

pub fn edit_with_builder<S>(data: S, builder: &Builder, cmd: Option<&Cmd>) -> Result<String>
where
	S: AsRef<[u8]>,
{
	match cmd {
		Some(cmd) => edit(&cmd.cmd, &cmd.args, data.as_ref(), builder),
		None => {
			let (path, args) = editor_cmd().ok_or_else(|| anyhow!("no editor could be found"))?;
			edit(&path, &args, data.as_ref(), builder)
		}
	}
}

fn edit<P: AsRef<Path>>(cmd: P, args: &[String], data: &[u8], builder: &Builder) -> Result<String> {
	let mut file = builder.tempfile()?;
	file.write_all(data)?;
	let path = file.into_temp_path();

	let status = Command::new(cmd.as_ref()).args(args).arg(&path).status()?;

	ensure!(status.success(), "editor command exited with {status}");

	let edited = fs::read_to_string(&path)?;
	path.close()?;

	Ok(edited)
}
