use anyhow::Context;
use chrono::Local;
use tokio::process::Command;

use crate::config::{Component, Config};

pub async fn get_status_text(config: &Config) -> anyhow::Result<String> {
    let mut parts = Vec::new();
    for part in config
        .components
        .iter()
        .map(|component| get_component_text(component, config))
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

async fn get_component_text(component: &Component, config: &Config) -> anyhow::Result<String> {
    Ok(match component {
        Component::Text(text) => text.clone(),
        Component::Separator { separator } => separator
            .clone()
            .unwrap_or_else(|| config.default_separator.clone()),
        Component::DateTime { format } => Local::now().format(format).to_string(),
        Component::Command { command } => {
            get_command_output(Command::new("sh").arg("-c").arg(command)).await?
        }
        Component::Playerctl { metadata_field } => {
            get_command_output(
                Command::new("playerctl")
                    .arg("metadata")
                    .arg(metadata_field),
            )
            .await?
        }
        _ => todo!(),
    })
}
