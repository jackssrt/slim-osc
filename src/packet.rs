use rosc::{OscMessage, OscPacket};
use tokio::net::UdpSocket;

pub async fn send_chat_message(socket: &UdpSocket, content: &str) -> anyhow::Result<()> {
    send_packet(
        socket,
        &OscPacket::Message(OscMessage {
            addr: "/chatbox/input".into(),
            args: vec![content.into(), true.into(), false.into()],
        }),
    )
    .await
}
async fn send_packet(socket: &UdpSocket, packet: &OscPacket) -> anyhow::Result<()> {
    let encoded = rosc::encoder::encode(packet)?;
    socket.send(&encoded).await?;
    Ok(())
}
