use tokio::time::Interval;

use crate::state::config::Config;

pub struct Timer {
    pub update_interval: Interval,
    pub update_count: u64,
}

impl Timer {
    pub fn new(config: &Config) -> Self {
        let mut update_interval = tokio::time::interval(config.update_interval);
        update_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        Self {
            update_interval,
            update_count: 0,
        }
    }
}
