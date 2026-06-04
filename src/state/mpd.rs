use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufStream},
    net::{TcpStream, ToSocketAddrs},
    sync::watch,
};
use tokio_util::sync::CancellationToken;
use tracing::{Level, instrument};

use crate::state::config::{Component, Config, MusicBackend, Source};

pub type Metadata = HashMap<Box<str>, Arc<str>>;

/// A socket that is connected to the MPD server and past the initial handshake.
struct Socket(BufStream<TcpStream>);

impl Socket {
    async fn new(address: impl ToSocketAddrs) -> anyhow::Result<Self> {
        let mut socket = TcpStream::connect(address).await.map(BufStream::new)?;

        // technically dont need to allocate a string on the heap here for this could just not care about the bytes at all
        // but this is faster to write
        let mut buf = String::with_capacity("OK MPD X.XX.X".len());
        socket.read_line(&mut buf).await?;
        tracing::trace!("handshake complete, {} is the greeting", buf.trim());

        Ok(Self(socket))
    }

    #[instrument(skip(self), level = Level::TRACE, ret, err(level = Level::ERROR))]
    pub async fn get_metadata(&mut self) -> anyhow::Result<Metadata> {
        let metadata = self
            .send_command("command_list_begin\nstatus\ncurrentsong\ncommand_list_end")
            .await?;
        let metadata = metadata
            .lines()
            .filter_map(|line| line.split_once(": "))
            .map(|(key, value)| (key.to_lowercase().into(), Arc::from(value)))
            .collect();
        Ok(metadata)
    }

    #[instrument(skip(self), level = Level::TRACE, ret, err(level = Level::ERROR))]
    async fn send_command(&mut self, packet: &str) -> anyhow::Result<String> {
        self.0.write_all(packet.as_bytes()).await?;
        // send a newline to show we're done
        self.0.write_all(&[0x0A]).await?;
        self.0.flush().await?;
        tracing::trace!("command sent");
        let mut buf = String::new();

        // read until we get "OK\n"
        loop {
            const END_PATTERN: &str = "OK\n";
            if self.0.read_line(&mut buf).await? == 0 {
                return Err(anyhow::anyhow!(
                    "connection closed while waiting for response"
                ));
            }

            if buf.ends_with(END_PATTERN) {
                buf.truncate(buf.len() - END_PATTERN.len());
                return Ok(buf);
            }
        }
    }
}

pub struct Mpd {
    pub metadata: watch::Receiver<Metadata>,
    pub cancel: CancellationToken,
}
impl Mpd {
    /// returns None if MPD isn't used in the config.
    pub async fn new(config: &Config) -> anyhow::Result<Option<Self>> {
        let MusicBackend::Mpd { address, port } = config.music_backend else {
            // #notmymusicbackend
            return Ok(None);
        };
        if !config.status.iter().any(|component| {
            matches!(
                component,
                Component::Interpolation {
                    source: Source::Music { .. },
                    ..
                }
            )
        }) {
            // music component not used
            return Ok(None);
        }

        tracing::debug!("hello world!");
        let cancel = CancellationToken::new();

        // setup the socket here so we have the data immediately
        let mut socket = Socket::new((address, port)).await?;
        tracing::debug!("connected");
        let metadata = socket.get_metadata().await?;
        let (tx, rx) = watch::channel(metadata);

        {
            let cancel = cancel.clone();
            tokio::spawn(Self::run(tx, socket, cancel));
        }
        Ok(Some(Self {
            metadata: rx,
            cancel,
        }))
    }

    #[instrument(skip(socket, tx), level = Level::TRACE, err(level = Level::ERROR))]
    async fn update_metadata(
        socket: &mut Socket,
        tx: &watch::Sender<Metadata>,
    ) -> anyhow::Result<()> {
        let metadata = socket.get_metadata().await?;
        tx.send_replace(metadata);
        Ok(())
    }

    #[instrument(skip_all, level = Level::TRACE, err(level = Level::ERROR))]
    async fn run(
        tx: watch::Sender<Metadata>,
        mut socket: Socket,
        cancel: CancellationToken,
    ) -> anyhow::Result<()> {
        loop {
            tokio::select! {
                Ok(_) = socket.send_command("idle") => {
                    Self::update_metadata(&mut socket, &tx).await?;
                }
                // if we get an abort signal, OR if the channel is closed we die
                () = cancel.cancelled() => {
                    tracing::debug!("bye world!");
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
