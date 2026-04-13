use std::{io::BufWriter, net::Ipv4Addr};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // open up connection
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await?;
    socket.connect("127.0.0.1:9000").await?;

    // send hardcoded message
    let packet = rosc::OscPacket::Message(rosc::OscMessage {
        addr: "/chatbox/input".into(),
        args: vec![
            format!("Hello world: {}", chrono::Utc::now().to_rfc3339()).into(),
            true.into(),
            false.into(),
        ],
    });
    let encoded = rosc::encoder::encode(&packet)?;
    socket.send(&encoded).await?;
    Ok(())
}
