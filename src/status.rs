use std::fmt::Display;

use chrono::Local;

use crate::config::{Component, Config};

pub async fn get_status_text(config: &Config) -> String {
    config
        .components
        .iter()
        .map(|component| match component {
            Component::Text { text } => text.clone(),
            Component::Separator { separator } => separator
                .clone()
                .unwrap_or_else(|| config.default_separator.clone()),
            Component::DateTime { format } => Local::now().format(format).to_string(),
            _ => todo!(),
        })
        .collect()
}
