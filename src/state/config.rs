use std::{net::IpAddr, path::Path, sync::Arc, time::Duration};

use serde::Deserialize;

use crate::state::config::status::Component;
pub mod status;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(deserialize_with = "status::deserialize")]
    pub status: Vec<Component>,
    #[serde(default = "default_address")]
    pub address: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_separator")]
    pub separator: Arc<str>,
    #[serde(default = "default_update_interval", alias = "refresh_interval")]
    pub update_interval: Duration,
    #[serde(default)]
    pub music_backend: MusicBackend,
}

fn default_address() -> IpAddr {
    "0.0.0.0"
        .parse()
        .expect("hardcoded ip should always be valid")
}
const fn default_port() -> u16 {
    9000
}

fn default_separator() -> Arc<str> {
    " - ".into()
}
const fn default_update_interval() -> Duration {
    Duration::from_secs(1)
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MusicBackend {
    // maybe replace with raw mpris later?
    #[cfg(target_os = "linux")]
    #[default]
    Playerctl,
    Mpd {
        #[serde(default = "default_mpd_address")]
        address: IpAddr,
        #[serde(default = "default_mpd_port")]
        port: u16,
    },
}

fn default_mpd_address() -> IpAddr {
    "127.0.0.1"
        .parse()
        .expect("hardcoded ip should always be valid")
}
const fn default_mpd_port() -> u16 {
    6600
}

impl Config {
    pub fn new(config_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(config_path)?;
        let config = toml::from_str(&content)?;
        tracing::trace!("config loaded: {:?}", config);
        Ok(config)
    }
}
