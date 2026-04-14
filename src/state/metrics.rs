use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::state::config::{Component, Config};
pub struct Metrics {
    pub system: System,
    refresh_kind: RefreshKind,
}
impl Metrics {
    fn calculate_refresh_kind(config: &Config) -> RefreshKind {
        // holy builder pattern
        let mut refresh_kind = RefreshKind::nothing();
        for component in &config.components {
            match component {
                Component::CpuUsage => {
                    refresh_kind =
                        refresh_kind.with_cpu(CpuRefreshKind::nothing().with_cpu_usage());
                }
                Component::MemoryUsage => {
                    refresh_kind =
                        refresh_kind.with_memory(MemoryRefreshKind::nothing().with_ram());
                }
                _ => {}
            }
        }
        refresh_kind
    }

    pub fn setup(config: &Config) -> Self {
        let refresh_kind = Self::calculate_refresh_kind(config);
        let mut system = System::new_with_specifics(refresh_kind);
        system.refresh_specifics(refresh_kind);

        Self {
            system,
            refresh_kind,
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_specifics(self.refresh_kind);
    }
}
