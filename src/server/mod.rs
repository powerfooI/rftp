use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub mod message;
pub mod user;

use crate::arg_parser::Args;

use self::user::TransferSession;

#[derive(Debug, Clone)]
pub struct Server {
  pub host: String,
  pub port: u16,
  pub root: String,
  pub listener: Arc<Mutex<TcpListener>>,
  pub user_map: Arc<Mutex<HashMap<SocketAddr, user::User>>>,
  pub data_map: Arc<Mutex<HashMap<u16, TransferSession>>>,
}

impl Server {
  pub async fn new(cfg: Args) -> Result<Self, tokio::io::Error> {
    let listener = TcpListener::bind(format!("{}:{}", cfg.host, cfg.port)).await?;

    Ok(Self {
      host: cfg.host,
      port: cfg.port,
      root: cfg.folder,
      listener: Arc::new(Mutex::new(listener)),
      user_map: Arc::new(Mutex::new(HashMap::new())),
      data_map: Arc::new(Mutex::new(HashMap::new())),
    })
  }

  #[allow(unused_must_use)]
  pub async fn listen(&self) {
    // todo: token pool and idle pool
    loop {
      let (socket, addr) = self.listener.lock().await.accept().await.unwrap();
      // let data_map = self.data_map.clone();
      let shared_self = self.clone();

      tokio::spawn(async move {
        shared_self.handle(socket, addr).await;
      });
    }
  }

  pub async fn handle(&self, mut socket: TcpStream, addr: SocketAddr) {
    let user_map = self.user_map.clone();
    loop {
      println!("addr {}", addr);
      if !user_map.lock().await.contains_key(&addr) {
        socket
          .write_all(b"220 rftp.whiteffire.cn FTP server ready.\0")
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
        let resp = self.dispatch(req, addr).await.unwrap();
        socket.write_all(resp.as_bytes()).await.unwrap();
        socket.flush().await.unwrap();
      }
    }
  }

  async fn dispatch(&self, req: String, addr: SocketAddr) -> Result<String, io::Error> {
    let cmd = message::parse_command(req);
    let user_map = self.user_map.clone();
    let mut user_map = user_map.lock().await;
    let current_user = user_map.get_mut(&addr).unwrap();
    match cmd {
      message::FtpCommand::USER(username) => {
        current_user.username = username;
        current_user.status = user::UserStatus::Logging;
        Ok("331 User name okay, need password.\0".to_string())
      }
      message::FtpCommand::PASS(password) => {
        println!("PASS: {}", password);
        if current_user.username == "anonymous" {
          current_user.status = user::UserStatus::Active;
          Ok("230 User logged in, proceed.\0".to_string())
        } else {
          Ok("530 Not logged in.".to_string())
        }
      }
      message::FtpCommand::PORT(addr) => {
        println!("PORT: {:?}", addr);
        current_user.status = user::UserStatus::Active;
        Ok("200 PORT command successful.".to_string())
      }
      message::FtpCommand::PASV(addr) => {
        println!("PASV: {:?}", addr);
        current_user.status = user::UserStatus::Active;
        Ok("200 PASV command successful.".to_string())
      }
      message::FtpCommand::RETR(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("150 File status okay; about to open data connection.".to_string())
      }
      message::FtpCommand::STOR(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("150 File status okay; about to open data connection.".to_string())
      }
      message::FtpCommand::ABOR => {
        current_user.status = user::UserStatus::Active;
        Ok("200 ABOR command successful.".to_string())
      }
      message::FtpCommand::QUIT => {
        user_map.remove(&addr);
        Ok("221 Goodbye.".to_string())
      }
      message::FtpCommand::SYST => Ok("215 UNIX Type: L8".to_string()),
      message::FtpCommand::TYPE => Ok("200 Type set to I.".to_string()),
      message::FtpCommand::RNFR(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("350 Requested file action pending further information.".to_string())
      }
      message::FtpCommand::RNTO(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("250 Requested file action okay, completed.".to_string())
      }
      message::FtpCommand::PWD => Ok(format!(
        "257 {} is the current directory.",
        current_user.pwd
      )),
      message::FtpCommand::CWD(dir) => {
        current_user.pwd = dir;
        Ok("250 Requested file action okay, completed.".to_string())
      }
      message::FtpCommand::MKD(dir) => {
        println!("user pwd: {} dir: {}", current_user.pwd, dir);
        Ok("257 Directory created.".to_string())
      }
      message::FtpCommand::RMD(dir) => {
        println!("user pwd: {} dir: {}", current_user.pwd, dir);
        Ok("250 Requested file action okay, completed.".to_string())
      }
      message::FtpCommand::LIST(optional_dir) => {
        println!(
          "user pwd: {}, path: {}",
          current_user.pwd,
          optional_dir.unwrap_or("unset".to_string())
        );
        Ok("150 Here comes the directory listing.".to_string())
      }
      message::FtpCommand::REST => {
        Ok("350 Requested file action pending further information.".to_string())
      }
      message::FtpCommand::DELE(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("250 Requested file action okay, completed.".to_string())
      }
      message::FtpCommand::STAT => Ok("211 End.".to_string()),
      message::FtpCommand::STOU => {
        Ok("202 Command not implemented, superfluous at this site.".to_string())
      }
      message::FtpCommand::APPE(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("150 File status okay; about to open data connection.".to_string())
      }
      message::FtpCommand::ALLO(size) => {
        // TODO: allocate disk space
        println!("Allocate disk space: {}", size);
        Ok("200 Command okay.".to_string())
      }
    }
  }
}
