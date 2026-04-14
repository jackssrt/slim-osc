use std::sync::Arc;

use anyhow::Context;
use arc_swap::ArcSwap;
use notify::Watcher;
use tokio::{sync::mpsc::Receiver, time::Interval};

use crate::{args::Args, config::Config};

pub fn try_reload(
    args: &Args,
    config: &ArcSwap<Config>,
    system: &mut sysinfo::System,
    interval: &mut Interval,
) {
    let Ok(new_config) = Config::new(args.config_path.clone()) else {
        tracing::error!("failed to read config, using old one...");
        return;
    };

    *interval = tokio::time::interval(new_config.update_interval);
    config.swap(Arc::new(new_config));
    *system = crate::setup_sysinfo(&config.load());
    tracing::info!("config reloaded successfully");
}

pub fn setup(args: &'static Args) -> anyhow::Result<Receiver<()>> {
    let (tx, rx) = tokio::sync::mpsc::channel(1);

    tracing::info!(
        "watching config file at {} for changes",
        args.config_path.display()
    );

    let mut watcher =
        notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            let file = event.expect("watcher error");
            if file.paths.iter().any(|path| path == &args.config_path) && file.kind.is_modify() {
                tracing::debug!("{:?}", file);
                let _ = tx.try_send(());
            }
        })?;

    watcher.watch(
        args.config_path
            .parent()
            .context("failed to get parent of config file")?,
        notify::RecursiveMode::NonRecursive,
    )?;

    Ok(rx)
}
