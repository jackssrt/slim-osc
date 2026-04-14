#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![forbid(unused_must_use)]

use anyhow::{Context, bail};
use clap::Parser;

use crate::{args::Args, state::State};

mod args;
mod hot_reloading;
mod state;
mod status;

async fn update_status(state: &mut State) -> anyhow::Result<()> {
    state.metrics.refresh();
    let status = status::get_status_text(state).await?;
    state.connection.send_chat_message(&status).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // setup logging
    tracing_subscriber::fmt::init();

    // parse args
    let args: &'static _ = Box::leak(Box::new(Args::parse()));

    // state
    let mut state = State::new(args).await?;

    // hot reloading
    let mut reload_rx = hot_reloading::setup(args).context("failed to setup hot reloading")?;

    loop {
        tokio::select! {
            _ = state.interval.tick() => {
                update_status(&mut state).await?;
            },
            Some(()) = reload_rx.recv() => {
                tracing::info!("config file changed, reloading...");
                hot_reloading::try_reload(args, &mut state).await;
            }
            else => bail!("broken channel"),
        }
    }
}
