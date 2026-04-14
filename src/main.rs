#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![forbid(unused_must_use)]
use std::sync::Arc;

use anyhow::{Context, bail};
use arc_swap::ArcSwap;
use clap::Parser;
use notify::Watcher;
use tokio::time::Interval;

use crate::{args::Args, config::Config, packet::send_chat_message, status::get_status_text};

mod args;
mod config;
mod connection;
mod packet;
mod status;

fn reload_config(
    args: &Args,
    config: &ArcSwap<Config>,
    interval: &mut Interval,
) -> anyhow::Result<()> {
    let new_config =
        Arc::new(Config::new(args.config_path.clone()).context("failed to read config")?);
    *interval = tokio::time::interval(new_config.update_interval);
    config.swap(new_config);
    tracing::info!("config reloaded successfully");
    Ok(())
}

async fn update_status(socket: &tokio::net::UdpSocket, config: &Config) -> anyhow::Result<()> {
    let status = get_status_text(config).await?;
    send_chat_message(socket, &status).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup logging
    tracing_subscriber::fmt::init();

    // parse args
    let args: &'static _ = Box::leak(Box::new(Args::parse()));
    let config = Config::new(&args.config_path).context("failed to read config")?;

    // open up connection
    let socket = connection::open(&config)
        .await
        .context("failed to open connection")?;

    // main loop
    let (reload_tx, mut reload_rx) = tokio::sync::mpsc::channel(1);
    let mut interval = tokio::time::interval(config.update_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let config = ArcSwap::new(Arc::new(config));

    // hot reloading
    tracing::info!(
        "watching config file at {} for changes",
        args.config_path.display()
    );
    let mut watcher =
        notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            let file = event.expect("watcher error");
            if file.paths.iter().any(|path| path == &args.config_path) && file.kind.is_modify() {
            	tracing::debug!("{:?}", file);
                let _ = reload_tx.try_send(());
            }
        })?;
    watcher.watch(
        args.config_path
            .parent()
            .context("failed to get parent of config file")?,
        notify::RecursiveMode::NonRecursive,
    )?;

    // main loop
    loop {
        tokio::select! {
            _ = interval.tick() => {
                update_status(&socket, &config.load()).await?;
            },
            Some(()) = reload_rx.recv() => {
                tracing::info!("config file changed, reloading...");
                reload_config(args, &config, &mut interval)?;
            }
            else => bail!("broken channel"),
        }
    }
}
