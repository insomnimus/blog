use std::{
	fmt,
	str::FromStr,
};

use anyhow::anyhow;
use serde::de::{
	self,
	Deserialize,
	Deserializer,
	Visitor,
};
use url::Url;

#[derive(Clone, Debug)]
pub struct SftpUri {
	pub remote: String,
	pub root: String,
	pub port: Option<u16>,
}

impl FromStr for SftpUri {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let url = s
			.parse::<Url>()
			.map_err(|e| anyhow!("could not parse the uri: {e}"))?;
		let host = url
			.host_str()
			.ok_or_else(|| anyhow!("missing the host field"))?;
		let remote = match url.username() {
			"" => host.to_string(),
			user => format!("{user}@{host}"),
		};
		let root = url.path().to_string();
		let port = url.port();
		Ok(Self { port, remote, root })
	}
}

struct UriVisitor;

impl<'de> Visitor<'de> for UriVisitor {
	type Value = SftpUri;

	fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_str("a string containing an sftp uri")
	}

	fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		s.parse::<SftpUri>().map_err(|e| E::custom(e.to_string()))
	}
}

impl<'de> Deserialize<'de> for SftpUri {
	fn deserialize<D>(des: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		des.deserialize_str(UriVisitor)
	}
}
