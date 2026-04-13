#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![forbid(unused_must_use)]
use anyhow::Context;

mod connection;
mod packet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // open up connection
    let socket = connection::open()
        .await
        .context("failed to open connection")?;

    // send packet
    Ok(())
}
