use std::io::{Result};
use tokio::{net::{TcpSocket}, io::AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<()> {
    let client = TcpSocket::new_v4()?;
    let mut stream = client.connect("127.0.0.1:8180".parse().unwrap()).await?;
    stream.write_all(b"200 ANONYMOUS").await?;
    Ok(())
}
