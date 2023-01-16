use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

pub mod message;
pub mod user;

use crate::arg_parser::Args;

#[derive(Debug)]
pub struct Server {
  pub host: String,
  pub port: u16,
  pub listener: TcpListener,
  pub user_map: Arc<Mutex<HashMap<SocketAddr, user::User>>>,
}

type JobReceiver = Arc<Mutex<mpsc::Receiver<message::SocketMsg>>>;

impl Server {
  pub async fn new(cfg: Args) -> Result<Self, tokio::io::Error> {
    let listener = TcpListener::bind(format!("{}:{}", cfg.host, cfg.port)).await?;

    Ok(Self {
      listener,
      host: cfg.host,
      port: cfg.port,
      user_map: Arc::new(Mutex::new(HashMap::new())),
    })
  }

  #[allow(unused_must_use)]
  pub async fn listen(&self) {
    // todo: token pool and idle pool

    let (tx, r) = mpsc::channel::<message::SocketMsg>(64);
    let ra = Arc::new(Mutex::new(r));

    // for i in 0..4 {
    //   let rx = ra.clone();
    //   let user_map = self.user_map.clone();

    //   tokio::spawn(async move {
    //     println!("worker {} set up", i);
    //     loop {
    //       let mut lock = rx.lock().await;
    //       println!("worker {} get lock", i);
    //       let msg = lock.recv().await.unwrap();
    //       println!("worker {}, message {:?}", i, &msg);
    //       let mut socket = msg.socket;
    //       println!("addr {}", &msg.addr);
    //       if !user_map.lock().await.contains_key(&msg.addr) {
    //         socket
    //           .write_all(b"220 rftp.whiteffire.cn FTP server ready.")
    //           .await
    //           .unwrap();

    //         user_map
    //           .lock()
    //           .await
    //           .insert(msg.addr.clone(), user::User::new_anonymous(msg.addr));
    //       }
    //     }
    //   });
    // }
    loop {
      let (mut socket, addr) = self.listener.accept().await.unwrap();
      let user_map = self.user_map.clone();
      tokio::spawn(async move {
        loop {
          println!("addr {}", addr);
          if !user_map.lock().await.contains_key(&addr) {
            socket
              .write_all(b"220 rftp.whiteffire.cn FTP server ready.")
              .await
              .unwrap();

            user_map
              .lock()
              .await
              .insert(addr.clone(), user::User::new_anonymous(addr));
          }

          loop {
            let mut buf = vec![0; 2048];
    
            let n = socket.read(&mut buf).await.unwrap();
            if n == 0 {
              return;
            }
            let req = String::from_utf8(buf).unwrap();
            println!("[Request] {}", &req);
          }
        }
      });
      // tx.send(message::SocketMsg {
      //   socket: Box::new(socket),
      //   // connection reset by peer
      //   addr,
      // });
    }
  }

  async fn handle_socket(&self, socket: TcpStream, addr: SocketAddr) -> Result<(), std::io::Error> {
    let user_map = self.user_map.clone();
    let (mut rd, mut wr) = io::split(socket);

    tokio::spawn(async move {
      println!("addr: {}", &addr);

      if !user_map.lock().await.contains_key(&addr) {
        wr.write_all(b"220 rftp.whiteffire.cn FTP server ready.")
          .await
          .unwrap_or_else(|_| println!("failed to send welcome message"));
        user_map
          .lock()
          .await
          .insert(addr.clone(), user::User::new_anonymous(addr));
      }

      loop {
        let mut buf = vec![0; 2048];

        let n = rd.read(&mut buf).await.unwrap();
        if n == 0 {
          return;
        }
        let req = String::from_utf8(buf).unwrap();
        println!("[Request] {}", &req);
      }
    })
    .await?;
    Ok(())
  }
}
