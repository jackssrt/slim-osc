use std::time::Duration;

use anyhow::Context;
use notify_debouncer_full::DebounceEventResult;
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

    let mut debouncer = notify_debouncer_full::new_debouncer(
        Duration::from_secs(2),
        Some(Duration::from_secs(2)),
        move |events: DebounceEventResult| {
            tracing::trace!("file events: {:?}", events);
            let events = events.expect("watcher error");
            if events
                .iter()
                // text editors often do "atomic swaps", where they make a temp file and rename it to *swap* it in place
                // imo this should just be built-in to the filesystem but whatever
                .any(|e| e.paths.contains(&args.config_path) && !e.kind.is_access())
            {
                tracing::debug!("{:?}", events);
                let _ = tx.try_send(());
            }
        },
    )?;

    debouncer.watch(
        args.config_path
            .parent()
            .context("failed to get parent of config file")?,
        notify::RecursiveMode::NonRecursive,
    )?;
    // the file watcher needs to be kept alive but we dont do anything with it after this, so just don't run its destructor
    std::mem::forget(debouncer);

    Ok(rx)
}
