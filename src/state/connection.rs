use std::time::Duration;

use rosc::{OscMessage, OscPacket};
use tokio::net::UdpSocket;
use tracing::{Level, instrument};

use crate::state::config::Config;

pub struct Connection {
    socket: UdpSocket,
}
impl Connection {
    pub async fn open(&Config { address, port, .. }: &Config) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        tracing::trace!("bound");
        for i in 1..=5 {
            match socket.connect((address, port)).await {
                Ok(()) => break,
                Err(e) => {
                    tracing::error!("failed to connect to {address}:{port} (attempt {i}): {e}");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
        tracing::info!("connected");
        Ok(Self { socket })
    }

    pub async fn send_chat_message(&self, content: &str) -> anyhow::Result<()> {
        self.send_packet(&OscPacket::Message(OscMessage {
            addr: "/chatbox/input".into(),
            args: vec![content.into(), true.into(), false.into()],
        }))
        .await
    }

    #[instrument(skip(self), level = Level::TRACE, ret, err(level = Level::ERROR))]
    async fn send_packet(&self, packet: &OscPacket) -> anyhow::Result<()> {
        let encoded = rosc::encoder::encode(packet)?;
        if self.socket.send(&encoded).await.is_err() {
            tracing::error!("failed to send packet");
        }
        Ok(())
    }
}
