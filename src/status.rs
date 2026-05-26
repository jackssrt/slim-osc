use std::sync::Arc;

use anyhow::Context;
use chrono::Local;
use tokio::process::Command;

use crate::state::{
    State,
    config::{
        MusicBackend,
        status::{Component, Filter, Source},
    },
    timer::Timer,
};

pub async fn get_status_text(state: &State) -> anyhow::Result<String> {
    let mut parts = Vec::new();
    for part in state
        .config
        .status
        .iter()
        .map(|component| component.eval(state))
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
impl Component {
    async fn eval(&self, state: &State) -> anyhow::Result<Arc<str>> {
        let text = match self {
            Self::Text(text) => text.clone(),
            Self::Interpolation { source, filters } => {
                let text = source.eval(state).await?;
                if filters.is_empty() {
                    return Ok(text);
                }
                let mut text = text.to_string();

                for filter in filters {
                    text = filter.eval(state, &text);
                }

                text.into()
            }
        };
        Ok(text)
    }
}

impl Source {
    async fn eval(
        &self,
        State {
            metrics,
            config,
            mpd,
            ..
        }: &State,
    ) -> anyhow::Result<Arc<str>> {
        Ok(match self {
            Self::Text(text) => text.clone(),
            Self::Separator => config.separator.clone(),
            Self::DateTime { format } => Local::now().format(format).to_string().into(),
            Self::Command { command } => {
                get_command_output(Command::new("sh").arg("-c").arg(command))
                    .await?
                    .into()
            }
            Self::Music {
                metadata: field_name,
            } => match config.music_backend {
                #[cfg(target_os = "linux")]
                MusicBackend::Playerctl => get_command_output(
                    Command::new("playerctl")
                        .arg("metadata")
                        .arg(field_name.to_string()),
                )
                .await?
                .into(),
                MusicBackend::Mpd { .. } => mpd
                    .as_ref()
                    .map(|mpd| {
                        mpd.metadata
                            .borrow()
                            .as_ref()
                            .and_then(|metadata| metadata.get(field_name).cloned())
                            .unwrap_or_else(|| "unknown".into())
                    })
                    .expect("mpd should be initialized if the music backend is mpd"),
            },
            // TODO: optimize model ones by caching them
            Self::CpuModel => metrics.system.cpus()[0].brand().into(),
            Self::CpuUsage => format!("{:.0}%", metrics.system.global_cpu_usage()).into(),
            Self::GpuModel => gfxinfo::active_gpu()
                .map_err(|_| anyhow::anyhow!("failed to get gpu"))?
                .model()
                .into(),
            Self::GpuUsage => format!(
                "{:}%",
                gfxinfo::active_gpu()
                    .map_err(|_| anyhow::anyhow!("failed to get gpu"))?
                    .info()
                    .load_pct()
            )
            .into(),
            #[allow(clippy::cast_precision_loss)]
            Self::MemoryUsage => format!(
                "{:.0}%",
                metrics.system.used_memory() as f64 / metrics.system.total_memory() as f64 * 100.0
            )
            .into(),
        })
    }
}

// TODO is this fast enough? would a hashmap be faster? someone better call the profiler
const SUPERSCRIPT: [(char, char); 64] = [
    // numbers
    ('0', '⁰'),
    ('1', '¹'),
    ('2', '²'),
    ('3', '³'),
    ('4', '⁴'),
    ('5', '⁵'),
    ('6', '⁶'),
    ('7', '⁷'),
    ('8', '⁸'),
    ('9', '⁹'),
    // lowercase letters
    ('a', 'ᵃ'),
    ('b', 'ᵇ'),
    ('c', 'ᶜ'),
    ('d', 'ᵈ'),
    ('e', 'ᵉ'),
    ('f', 'ᶠ'),
    ('g', 'ᵍ'),
    ('h', 'ʰ'),
    ('i', 'ᶦ'),
    ('j', 'ʲ'),
    ('k', 'ᵏ'),
    ('l', 'ˡ'),
    ('m', 'ᵐ'),
    ('n', 'ⁿ'),
    ('o', 'ᵒ'),
    ('p', 'ᵖ'),
    ('q', '𐞥'),
    ('r', 'ʳ'),
    ('s', 'ˢ'),
    ('t', 'ᵗ'),
    ('u', 'ᵘ'),
    ('v', 'ᵛ'),
    ('w', 'ʷ'),
    ('x', 'ˣ'),
    ('y', 'ʸ'),
    ('z', 'ᶻ'),
    // uppercase letters
    ('A', 'ᴬ'),
    ('B', 'ᴮ'),
    ('C', 'ꟲ'),
    ('D', 'ᴰ'),
    ('E', 'ᴱ'),
    ('F', 'ꟳ'),
    ('G', 'ᴳ'),
    ('H', 'ᴴ'),
    ('I', 'ᴵ'),
    ('J', 'ᴶ'),
    ('K', 'ᴷ'),
    ('L', 'ᴸ'),
    ('M', 'ᴹ'),
    ('N', 'ᴺ'),
    ('O', 'ᴼ'),
    ('P', 'ᴾ'),
    ('Q', 'ꟴ'),
    ('R', 'ᴿ'),
    ('S', '꟱'),
    ('T', 'ᵀ'),
    ('U', 'ᵁ'),
    ('V', 'ⱽ'),
    ('W', 'ᵂ'), // xyz are missing, wikipedia
    // symbols
    ('+', '⁺'),
    ('-', '⁻'),
    ('=', '⁼'),
    ('(', '⁽'),
    (')', '⁾'),
];
const SUBSCRIPT: [(char, char); 31] = [
    // numbers
    ('0', '₀'),
    ('1', '₁'),
    ('2', '₂'),
    ('3', '₃'),
    ('4', '₄'),
    ('5', '₅'),
    ('6', '₆'),
    ('7', '₇'),
    ('8', '₈'),
    ('9', '₉'),
    // lowercase letters
    ('a', 'ₐ'),
    ('e', 'ₑ'),
    ('h', 'ₕ'),
    ('i', 'ᵢ'),
    ('j', 'ⱼ'),
    ('k', 'ₖ'),
    ('l', 'ₗ'),
    ('m', 'ₘ'),
    ('n', 'ₙ'),
    ('o', 'ₒ'),
    ('p', 'ₚ'),
    ('r', 'ᵣ'),
    ('s', 'ₛ'),
    ('t', 'ₜ'),
    ('u', 'ᵤ'),
    ('v', 'ᵥ'),
    ('x', 'ₓ'),
    // symbols
    ('+', '₊'),
    ('-', '₋'),
    ('(', '₍'),
    (')', '₎'),
];

impl Filter {
    fn eval(
        &self,
        State {
            timer: Timer { update_count, .. },
            ..
        }: &State,
        input: &str,
    ) -> String {
        match self {
            Self::Uppercase => input.to_uppercase(),
            Self::Lowercase => input.to_lowercase(),
            Self::Trim => input.trim().to_string(),
            Self::Subscript => input
                .chars()
                .map(|c| {
                    SUBSCRIPT
                        .iter()
                        .find(|(normal, _)| *normal == c)
                        .map_or(c, |(_, sub)| *sub)
                })
                .collect::<String>(),
            Self::Superscript => input
                .chars()
                .map(|c| {
                    SUPERSCRIPT
                        .iter()
                        .find(|(normal, _)| *normal == c)
                        .map_or(c, |(_, sup)| *sup)
                })
                .collect::<String>(),
            Self::Marquee {
                length: target_length,
                period,
            } => {
                let length = input.chars().count();
                if *target_length >= length {
                    return input.to_string();
                }

                let chars = input.chars().collect::<Vec<char>>();
                let mut windows = chars.windows(*target_length);
                // user has bigger problems if theyre living before 1970

                #[allow(clippy::cast_sign_loss)]
                #[allow(clippy::cast_possible_truncation)]
                #[allow(clippy::cast_precision_loss)]
                {
                    // double the period to account for the time to go back and forth
                    let time_over_period = *update_count as f64 / period;
                    let offset = time_over_period % 1.0; // oscillate between 0 and 1
                    let offset = offset * (windows.len() - 1) as f64; // scale to possible windows indexes
                    windows
                        .nth(offset.round() as usize)
                        .expect(
                            "windows should have at least one element since target_length is > 0",
                        )
                        .iter()
                        .collect()
                }
            }
            Self::Truncate { length } => {
                if input.chars().count() > *length {
                    input
                        .chars()
                        .take(*length - 1)
                        .chain(std::iter::once('…'))
                        .collect()
                } else {
                    input.to_string()
                }
            }
        }
    }
}
