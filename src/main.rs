#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![forbid(unused_must_use)]
use std::sync::Arc;

use anyhow::{Context, bail};
use arc_swap::ArcSwap;
use clap::Parser;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::{
    args::Args,
    config::{Component, Config},
    packet::send_chat_message,
    status::get_status_text,
};

mod args;
mod config;
mod connection;
mod hot_reloading;
mod packet;
mod status;

fn get_refresh_kind(config: &Config) -> RefreshKind {
    // holy builder pattern
    let mut refresh_kind = RefreshKind::nothing();
    for component in &config.components {
        match component {
            Component::CpuUsage => {
                refresh_kind = refresh_kind.with_cpu(CpuRefreshKind::nothing().with_cpu_usage());
            }
            Component::MemoryUsage => {
                refresh_kind = refresh_kind.with_memory(MemoryRefreshKind::nothing().with_ram());
            }
            _ => {}
        }
    }
    refresh_kind
}

async fn update_status(
    socket: &tokio::net::UdpSocket,
    config: &Config,
    system: &mut System,
) -> anyhow::Result<()> {
    system.refresh_specifics(get_refresh_kind(config));
    let status = get_status_text(config, system).await?;
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
    let refresh_kind = get_refresh_kind(&config);

    // open up connection
    let socket = connection::open(&config)
        .await
        .context("failed to open connection")?;

    // main loop
    let mut interval = tokio::time::interval(config.update_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let config = ArcSwap::new(Arc::new(config));

    let mut reload_rx = hot_reloading::setup(args).context("failed to setup hot reloading")?;

    // main loop
    let mut system = System::new_with_specifics(refresh_kind);
    system.refresh_specifics(refresh_kind);
    loop {
        tokio::select! {
            _ = interval.tick() => {
                update_status(&socket, &config.load(), &mut system).await?;
            },
            Some(()) = reload_rx.recv() => {
                tracing::info!("config file changed, reloading...");
                hot_reloading::try_reload(args, &config, &mut interval);
            }
            else => bail!("broken channel"),
        }
    }
}
