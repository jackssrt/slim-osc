use std::sync::Arc;

use chrono::Local;
use tokio::process::Command;
#[cfg(target_os = "windows")]
use windows::Media::Control::{
    // literally have to come up with my own names because the windows api is so bad
    GlobalSystemMediaTransportControlsSessionManager as MediaControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as MediaControlsPlaybackStatus,
};

use crate::state::{State, config::MusicBackend};

mod helpers {
    use std::sync::Arc;

    use anyhow::Context;
    use cached::once;
    use tokio::process::Command;

    use crate::state::metrics::Metrics;

    pub(super) async fn get_command_output(command: &mut Command) -> anyhow::Result<String> {
        command
            .output()
            .await
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .context("output command failed")
    }

    #[once]
    pub(super) fn get_gpu_model() -> Option<Arc<str>> {
        gfxinfo::active_gpu().map(|gpu| gpu.model().into()).ok()
    }

    #[once]
    pub(super) fn get_cpu_model(metrics: &Metrics) -> Arc<str> {
        metrics.system.cpus()[0].brand().into()
    }
}

#[derive(Debug, Clone)]
#[deny(dead_code)] // use it in the parser
pub enum MusicProperty {
    Status,
    Metadata(Arc<str>),
}

#[derive(Debug, Clone)]
#[deny(dead_code)] // use it in the parser
pub enum Source {
    Separator,
    DateTime {
        format: String,
    },

    // Usage
    GpuUsage,
    CpuUsage,
    MemoryUsage,

    // Model
    GpuModel,
    CpuModel,

    Music(MusicProperty),

    Command {
        // this is a string because its more efficient to use it later, can't convert a smart pointer to str to an &OsStr grr
        command: String,
    },
    Text(Arc<str>),
}
impl Source {
    pub(super) async fn eval(
        &self,
        State {
            metrics,
            config,
            mpd,
            ..
        }: &State,
    ) -> anyhow::Result<Arc<str>> {
        let text: Option<_> = match self {
            Self::Text(text) => Some(text.clone()),
            Self::Separator => Some(config.separator.clone()),
            Self::DateTime { format } => Some(Local::now().format(format).to_string().into()),
            Self::Command { command } => {
                let (executable, flag) = if cfg!(target_os = "windows") {
                    ("powershell", "-Command")
                } else {
                    ("sh", "-c")
                };
                Some(
                    helpers::get_command_output(Command::new(executable).arg(flag).arg(command))
                        .await?
                        .into(),
                )
            }

            Self::Music(music_property) => match config.music_backend {
                #[cfg(target_os = "linux")]
                MusicBackend::Playerctl => {
                    let mut command = Command::new("playerctl");
                    Some(
                        helpers::get_command_output(match music_property {
                            MusicProperty::Status => command.arg("status"),
                            MusicProperty::Metadata(field_name) => {
                                command.arg("metadata").arg(field_name.to_string())
                            }
                        })
                        .await?
                        .into(),
                    )
                }
                MusicBackend::Mpd { .. } => mpd
                    .as_ref()
                    .expect("mpd should be initialized if the music backend is mpd")
                    .metadata
                    .borrow()
                    .get(match music_property {
                        MusicProperty::Status => "state",
                        MusicProperty::Metadata(field_name) => field_name.as_ref(),
                    })
                    .cloned(),
                #[cfg(target_os = "windows")]
                MusicBackend::MediaSession => {
                    let manager = MediaControlsSessionManager::RequestAsync()
                        .context("failed to get media session manager")?
                        .await
                        .context("failed to get media session manager")?;

                    let session = manager.GetCurrentSession()?;

                    match music_property {
                        MusicProperty::Status => {
                            match session.GetPlaybackInfo()?.PlaybackStatus()? {
                                MediaControlsPlaybackStatus::Playing => Some("playing".into()),
                                MediaControlsPlaybackStatus::Paused => Some("paused".into()),
                                MediaControlsPlaybackStatus::Stopped => Some("stopped".into()),
                                _ => None,
                            }
                        }
                        MusicProperty::Metadata(field_name) => {
                            let media_properties = session.TryGetMediaPropertiesAsync()?.await?;
                            match field_name.as_ref() {
                                "title" => media_properties
                                    .Title()
                                    .ok()
                                    .map(|x| x.to_string_lossy().into()),
                                "artist" => media_properties
                                    .Artist()
                                    .ok()
                                    .map(|x| x.to_string_lossy().into()),
                                "album" => media_properties
                                    .AlbumTitle()
                                    .ok()
                                    .map(|x| x.to_string_lossy().into()),
                                _ => None,
                            }
                        }
                    }
                }
            },
            Self::CpuModel => Some(helpers::get_cpu_model(metrics)),
            Self::CpuUsage => Some(format!("{:.0}%", metrics.system.global_cpu_usage()).into()),
            Self::GpuModel => helpers::get_gpu_model(),
            Self::GpuUsage => gfxinfo::active_gpu()
                .map(|gpu| format!("{:.0}%", gpu.info().load_pct()).into())
                .ok(),
            #[allow(clippy::cast_precision_loss)]
            Self::MemoryUsage => Some(
                format!(
                    "{:.0}%",
                    metrics.system.used_memory() as f64 / metrics.system.total_memory() as f64
                        * 100.0
                )
                .into(),
            ),
        };

        Ok(text.unwrap_or_else(|| "unknown".into()))
    }
}
