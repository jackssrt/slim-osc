use std::{fs, net::IpAddr, path::Path, sync::Arc, time::Duration};

use anyhow::Context;
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
    #[default]
    #[cfg(target_os = "windows")]
    MediaSession,
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
    pub fn new(config_path: &Path) -> anyhow::Result<Self> {
        let content = match std::fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // slight security vulnerability here since someone could invoke us with an arbitrary config path
                // and make us create empty files somewhere we're not supposed to, meh

                // can't make a default config (custom components deserializer prevents us from being able to serialize a Config)
                // but will at least touch the file
                tracing::warn!(
                    "config file not found at {}, trying to make an empty one",
                    config_path.display()
                );

                // looks weird but this will call fs::create_dir with the parent directory if it exists
                // and ignore errors
                fs::create_dir_all(
                    config_path
                        .parent()
                        .context("failed to get parent directory of config file")?,
                )
                .context("failed to create parent directories of config file")?;

                // okay enjoy
                fs::File::create_new(config_path).context("failed to create config file")?;

                return Err(e).context(format!(
                    "config file not found at {}",
                    config_path.display()
                ));
            }
            Err(e) => {
                return Err(e).context(format!(
                    "failed to read config at {}",
                    config_path.display()
                ));
            }
        };

        let config = toml::from_str(&content).context("failed to parse config")?;
        tracing::trace!("config loaded: {:?}", config);
        Ok(config)
    }
}
