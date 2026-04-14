#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![forbid(unused_must_use)]
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::{config::Config, packet::send_chat_message};

mod config;
mod connection;
mod packet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO: this is linux only
    let xdg_config: PathBuf = std::env::var("XDG_CONFIG_HOME")?.into();
    let config =
        Config::new(xdg_config.join("slim-osc/config.toml")).context("failed to read config")?;

    // open up connection
    let socket = connection::open(&config)
        .await
        .context("failed to open connection")?;

    // send packet
    Ok(())
}
