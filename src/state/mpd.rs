use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{TcpStream, ToSocketAddrs},
    sync::watch,
};
use tokio_util::sync::CancellationToken;

use crate::state::config::{Config, MusicBackend};

pub type Metadata = HashMap<Arc<str>, Arc<str>>;

/// A socket that is connected to the MPD server and past the initial handshake.
struct Socket(BufStream<TcpStream>);

impl Socket {
    async fn new(address: impl ToSocketAddrs) -> anyhow::Result<Self> {
        let mut socket = TcpStream::connect(address).await.map(BufStream::new)?;

        // technically dont need to allocate a string on the heap here for this could just not care about the bytes at all
        // but this is faster to write
        socket
            .read_line(&mut String::with_capacity("OK MPD X.XX.X".len()))
            .await?;

        Ok(Self(socket))
    }

    pub async fn get_metadata(&mut self) -> anyhow::Result<Metadata> {
        let metadata = self.send_command("currentsong").await?;
        let metadata = metadata
            .lines()
            .filter_map(|line| line.split_once(": "))
            .map(|(key, value)| (Arc::from(key), Arc::from(value)))
            .collect();
        Ok(metadata)
    }

    async fn send_command(&mut self, packet: &str) -> anyhow::Result<String> {
        self.0.write_all(packet.as_bytes()).await?;
        let mut buf = String::new();

        // read until we get "OK\n"
        loop {
            const END_PATTERN: &str = "OK\n";
            self.0.read_line(&mut buf).await?;

            if buf.ends_with(END_PATTERN) {
                buf.truncate(buf.len() - END_PATTERN.len());
                return Ok(buf);
            }
        }
    }
}

pub struct Mpd {
    pub metadata: watch::Receiver<Option<Metadata>>,
    pub cancel: CancellationToken,
}
impl Mpd {
    /// returns None if MPD isn't used in the config.
    pub fn new(config: &Config) -> Option<Self> {
        let MusicBackend::Mpd { address, port } = config.music_backend else {
            // #notmymusicbackend
            return None;
        };
        if !config
            .components
            .iter()
            .any(|component| matches!(component, crate::state::config::Component::Music { .. }))
        {
            // music component not used
            return None;
        }

        let (tx, rx) = watch::channel(None);
        let cancel = CancellationToken::new();
        {
            let cancel = cancel.clone();
            tokio::spawn(Self::run(tx, cancel, (address, port)));
        }
        Some(Self {
            metadata: rx,
            cancel,
        })
    }

    async fn run(
        tx: watch::Sender<Option<Metadata>>,
        cancel: CancellationToken,
        address: impl ToSocketAddrs,
    ) -> anyhow::Result<()> {
        let mut socket = Socket::new(address).await?;
        loop {
            tokio::select! {
                Ok(_) = socket.send_command("idle") => {
                    let metadata = socket.get_metadata().await?;
                    tx.send_replace(Some(metadata));
                }
                // if we get an abort signal, OR if the channel is closed we die
                () = cancel.cancelled() => {
                    return Ok(());
                }
            }
        }
    }
}

impl Drop for Mpd {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
