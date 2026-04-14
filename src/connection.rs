use tokio::net::UdpSocket;

use crate::config::Config;

pub async fn open(&Config { address, port, .. }: &Config) -> anyhow::Result<UdpSocket> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(format!("{address}:{port}")).await?;
    Ok(socket)
}
