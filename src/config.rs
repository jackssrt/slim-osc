use std::{net::IpAddr, path::Path, time::Duration};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub components: Vec<Component>,
    #[serde(default = "default_address")]
    pub address: IpAddr,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_separator")]
    pub default_separator: String,
    #[serde(default = "default_update_interval")]
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

fn default_separator() -> String {
    " - ".to_string()
}
const fn default_update_interval() -> Duration {
    Duration::from_secs(2)
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Component {
    #[serde(alias = "sep")]
    Separator {
        separator: Option<String>,
    },
    #[serde(alias = "time", alias = "date")]
    DateTime {
        format: String,
    },

    // Usage
    GpuUsage,
    CpuUsage,
    MemoryUsage,

    // Model
    GpuModel,
    CpuModel,

    Music {
        #[serde(alias = "metadata")]
        metadata_field: String,
    },

    #[serde(untagged)]
    Command {
        command: String,
    },

    #[serde(untagged)]
    Text(String),
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum MusicBackend {
    // maybe replace with raw mpris later?
    #[cfg(target_os = "linux")]
    #[default]
    Playerctl,
    Mpd,
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        tracing::trace!("config loaded: {:?}", config);
        Ok(config)
    }
}
