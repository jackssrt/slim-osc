use rosc::{OscMessage, OscPacket};
use tokio::net::UdpSocket;

pub async fn send_chat_message(socket: &UdpSocket) -> anyhow::Result<()> {
    send_packet(
        socket,
        &OscPacket::Message(OscMessage {
            addr: "/chatbox/input".into(),
            args: vec![
                format!("Hello world: {}", chrono::Utc::now().to_rfc3339()).into(),
                true.into(),
                false.into(),
            ],
        }),
    ).await
}
async fn send_packet(socket: &UdpSocket, packet: &OscPacket) -> anyhow::Result<()> {
    let encoded = rosc::encoder::encode(packet)?;
    socket.send(&encoded).await?;
    Ok(())
}
