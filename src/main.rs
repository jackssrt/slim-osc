#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![forbid(unused_must_use)]
use std::path::{Path, PathBuf};

use anyhow::Context;
use tokio::time::{Interval, interval};

use crate::{config::Config, packet::send_chat_message, status::get_status_text};

mod config;
mod connection;
mod packet;
mod status;

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
    let status = get_status_text(&config);
    send_chat_message(&socket, &status).await?;

    Ok(())
}
