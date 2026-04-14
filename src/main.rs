#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![forbid(unused_must_use)]
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

use crate::{args::Args, config::Config, packet::send_chat_message, status::get_status_text};

mod args;
mod config;
mod connection;
mod packet;
mod status;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // parse args
    let args = Args::parse();
    // parse config
    let config = Config::new(args.config_path).context("failed to read config")?;

    // open up connection
    let socket = connection::open(&config)
        .await
        .context("failed to open connection")?;

    // main loop
    let mut interval = tokio::time::interval(config.update_interval);
    loop {
        interval.tick().await;
        let status = get_status_text(&config);
        send_chat_message(&socket, &status).await?;
    }
}
