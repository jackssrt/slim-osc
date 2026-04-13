use tokio::net::UdpSocket;

pub async fn open() -> anyhow::Result<UdpSocket> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    socket.connect("127.0.0.1:9000").await?;
    Ok(socket)
}
