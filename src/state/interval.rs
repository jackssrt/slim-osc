use crate::state::config::Config;

pub fn new(config: &Config) -> tokio::time::Interval {
    let mut interval = tokio::time::interval(config.update_interval);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval
}
