use anyhow::Context;
use notify::Watcher;
use tokio::sync::mpsc::Receiver;

use crate::{args::Args, state::State};

pub async fn try_reload(args: &'static Args, state: &mut State) {
    let Ok(new_state) = State::new(args).await else {
        tracing::error!("failed to read config, using old one...");
        return;
    };

    *state = new_state;
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
