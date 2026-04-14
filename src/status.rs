use anyhow::Context;
use chrono::Local;
use sysinfo::System;
use tokio::process::Command;

use crate::config::{Component, Config, MusicBackend};

pub async fn get_status_text(config: &Config, system: &System) -> anyhow::Result<String> {
    let mut parts = Vec::new();
    for part in config
        .components
        .iter()
        .map(|component| get_component_text(component, config, system))
    {
        parts.push(part.await?);
    }
    Ok(parts.join(""))
}
async fn get_command_output(command: &mut Command) -> anyhow::Result<String> {
    command
        .output()
        .await
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .context("output command failed")
}

async fn get_component_text(
    component: &Component,
    config: &Config,
    system: &System,
) -> anyhow::Result<String> {
    Ok(match component {
        Component::Text(text) => text.clone(),
        Component::Separator { separator } => separator
            .clone()
            .unwrap_or_else(|| config.default_separator.clone()),
        Component::DateTime { format } => Local::now().format(format).to_string(),
        Component::Command { command } => {
            get_command_output(Command::new("sh").arg("-c").arg(command)).await?
        }
        Component::Music { metadata_field } => match config.music_backend {
            #[cfg(target_os = "linux")]
            MusicBackend::Playerctl => {
                get_command_output(
                    Command::new("playerctl")
                        .arg("metadata")
                        .arg(metadata_field),
                )
                .await?
            }
            MusicBackend::Mpd => {
                get_command_output(
                    Command::new("mpc")
                        .arg("-f")
                        .arg(format!("%{metadata_field}%"))
                        .arg("current"),
                )
                .await?
            }
        },
        Component::CpuModel => system.cpus()[0].brand().to_string(),
        Component::CpuUsage => format!("{:.0}%", system.global_cpu_usage()),
        Component::GpuModel => gfxinfo::active_gpu()
            .map_err(|_| anyhow::anyhow!("failed to get gpu"))?
            .model()
            .to_string(),
        Component::GpuUsage => format!(
            "{:}%",
            gfxinfo::active_gpu()
                .map_err(|_| anyhow::anyhow!("failed to get gpu"))?
                .info()
                .load_pct()
        ),
        #[allow(clippy::cast_precision_loss)]
        Component::MemoryUsage => format!(
            "{:.0}%",
            system.used_memory() as f64 / system.total_memory() as f64 * 100.0
        ),
    })
}
