use std::str::FromStr;

use anyhow::anyhow;
use serde::{
	de::{
		self,
		Deserializer,
	},
	Deserialize,
};

#[derive(Debug, Clone)]
pub struct Cmd {
	pub cmd: String,
	pub args: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum SerCmd {
	Str(String),
	Arr(Vec<String>),
}

impl Cmd {
	pub fn to_tokio(&self) -> tokio::process::Command {
		let mut c = tokio::process::Command::new(&self.cmd);
		c.args(&self.args);
		c
	}

	pub fn to_std(&self) -> std::process::Command {
		let mut c = std::process::Command::new(&self.cmd);
		c.args(&self.args);
		c
	}
}

impl FromStr for Cmd {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut words = shell_words::split(s)?;
		if words.is_empty() {
			Err(anyhow!("command is empty"))
		} else {
			let cmd = words.remove(0);
			Ok(Self { cmd, args: words })
		}
	}
}

impl<'de> Deserialize<'de> for Cmd {
	fn deserialize<D>(des: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		match SerCmd::deserialize(des)? {
			SerCmd::Str(s) => s.parse::<Self>().map_err(de::Error::custom),
			SerCmd::Arr(v) if v.is_empty() => Err(de::Error::custom("the command is empty")),
			SerCmd::Arr(mut v) => {
				let cmd = v.remove(0);
				Ok(Self { cmd, args: v })
			}
		}
	}
}
