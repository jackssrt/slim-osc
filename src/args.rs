use std::path::PathBuf;

use clap::Parser;

fn default_config_path() -> PathBuf {
    // TODO: this is linux only
    let config_home: PathBuf = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| "~/.config".into())
        .into();
    config_home.join("slim-osc/config.toml")
}

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, default_value=default_config_path().into_os_string())]
    pub config_path: PathBuf,
}
