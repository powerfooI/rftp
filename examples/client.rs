use std::io::{stdin, Result};
use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  net::TcpSocket,
};

#[tokio::main]
async fn main() -> Result<()> {
  let client = TcpSocket::new_v4()?;
  let stream = client.connect("127.0.0.1:8180".parse().unwrap()).await?;

  let (mut rd, mut wr) = io::split(stream);

  let read_task = tokio::spawn(async move {
    loop {
      let mut input = String::new();
      stdin().read_line(&mut input).unwrap();
      wr.write_all(input.as_bytes()).await.unwrap();
    }
  });

  let write_task = tokio::spawn(async move {
    let mut buf = vec![0u8; 1024];
    loop {
      let n = rd.read(&mut buf).await.unwrap();
      if n == 0 {
        break;
      }
      println!("{}", String::from_utf8_lossy(&buf[..n]));
    }
  });

  read_task.await?;
  write_task.await?;
  
  Ok(())
}
