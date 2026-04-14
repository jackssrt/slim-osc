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
        socket.connect(format!("{address}:{port}")).await?;
        tracing::info!("connected");
        Ok(Self { socket })
    }

    #[instrument(skip(self), level = Level::TRACE, ret)]
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
        self.socket.send(&encoded).await?;
        Ok(())
    }
}
