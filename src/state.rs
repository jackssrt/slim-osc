pub mod config;
pub mod connection;
mod interval;
mod metrics;

use anyhow::Context;

use crate::{
    args::Args,
    state::{config::Config, connection::Connection, metrics::Metrics},
};

pub struct State {
    pub config: Config,
    pub connection: Connection,
    pub metrics: Metrics,
    pub interval: tokio::time::Interval,
}
impl State {
    pub async fn new(args: &'static Args) -> anyhow::Result<Self> {
        let config = Config::new(&args.config_path).context("failed to read config")?;
        Ok(Self {
            connection: Connection::open(&config).await?,
            metrics: Metrics::setup(&config),
            interval: interval::new(&config),
            config,
        })
    }
}
