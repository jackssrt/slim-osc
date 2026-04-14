use std::fmt::Display;

use crate::config::{Component, Config};

pub fn get_status_text(config: &Config) -> String {
    config
        .components
        .iter()
        .map(|component| match component {
            Component::Text { text } => text.clone(),
            Component::Separator { separator } => separator
                .clone()
                .unwrap_or_else(|| config.default_separator.clone()),
            _ => todo!(),
        })
        .collect()
}
