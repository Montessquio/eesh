
use std::path::Path;
use color_eyre::Result;
use hashbrown::HashMap;
use irc::client::prelude::*;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ClientConfig {
    pub default_quit: Option<String>,

    #[serde(flatten)]
    pub irc: Config,
}

impl ClientConfig {
    pub fn parse_str(raw: &str) -> Result<HashMap<String, ClientConfig>> {
        Ok(toml::from_str(raw)?)
    }

    pub fn parse(path: impl AsRef<Path>) -> Result<HashMap<String, ClientConfig>> {
        Self::parse_str(&std::fs::read_to_string(path.as_ref())?)
    }
}
