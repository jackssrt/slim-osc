use std::sync::Arc;

use anyhow::Context;
use chrono::Local;
use tokio::process::Command;

use crate::state::{
    State,
    config::{Component, MusicBackend},
};

pub async fn get_status_text(state: &State) -> anyhow::Result<String> {
    let mut parts = Vec::new();
    for part in state
        .config
        .components
        .iter()
        .map(|component| get_component_text(state, component))
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
    State {
        metrics,
        config,
        mpd,
        ..
    }: &State,
    component: &Component,
) -> anyhow::Result<Arc<str>> {
    Ok(match component {
        Component::Text(text) => text.clone(),
        Component::Separator { separator } => separator
            .clone()
            .unwrap_or_else(|| config.default_separator.clone()),
        Component::DateTime { format } => Local::now().format(format).to_string().into(),
        Component::Command { command } => {
            get_command_output(Command::new("sh").arg("-c").arg(command))
                .await?
                .into()
        }
        Component::Music { metadata_field } => match config.music_backend {
            #[cfg(target_os = "linux")]
            MusicBackend::Playerctl => get_command_output(
                Command::new("playerctl")
                    .arg("metadata")
                    .arg(metadata_field.to_string()),
            )
            .await?
            .into(),
            MusicBackend::Mpd { .. } => mpd
                .as_ref()
                .map(|mpd| {
                    mpd.metadata
                        .borrow()
                        .as_ref()
                        .and_then(|metadata| metadata.get(metadata_field).cloned())
                        .unwrap_or_else(|| "unknown".into())
                })
                .expect("mpd should be initialized if the music backend is mpd"),
        },
        // TODO: optimize model ones by caching them
        Component::CpuModel => metrics.system.cpus()[0].brand().into(),
        Component::CpuUsage => format!("{:.0}%", metrics.system.global_cpu_usage()).into(),
        Component::GpuModel => gfxinfo::active_gpu()
            .map_err(|_| anyhow::anyhow!("failed to get gpu"))?
            .model()
            .into(),
        Component::GpuUsage => format!(
            "{:}%",
            gfxinfo::active_gpu()
                .map_err(|_| anyhow::anyhow!("failed to get gpu"))?
                .info()
                .load_pct()
        )
        .into(),
        #[allow(clippy::cast_precision_loss)]
        Component::MemoryUsage => format!(
            "{:.0}%",
            metrics.system.used_memory() as f64 / metrics.system.total_memory() as f64 * 100.0
        )
        .into(),
    })
}
