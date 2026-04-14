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
}

fn default_address() -> IpAddr {
    "0.0.0.0"
        .parse()
        .expect("hardcoded ip should always be valid")
}
fn default_port() -> u16 {
    9000
}

fn default_separator() -> String {
    " - ".to_string()
}
fn default_update_interval() -> Duration {
    Duration::from_secs(2)
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Component {
    #[serde(alias = "sep")]
    Separator {
        separator: Option<String>,
    },
    Text {
        text: String,
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

    Output {
        command: String,
    },
}

impl Config {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
