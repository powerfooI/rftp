use std::io::{Result, stdout};
use tokio::{
  io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
  net::TcpSocket,
};

#[tokio::main]
async fn main() -> Result<()> {
  let client = TcpSocket::new_v4()?;
  let stream = client.connect("127.0.0.1:8180".parse().unwrap()).await?;

  let (mut rd, mut wr) = io::split(stream);
  wr.write_all(b"200 Anonymous").await?;
  
  let mut buf = vec![0u8; 1024];
  
  loop {
    let n = rd.read(&mut buf).await?;
    if n == 0 {
      break;
    }
    println!("{}", String::from_utf8(buf.clone()).unwrap());
  }

  Ok(())
}
