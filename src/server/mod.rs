use chrono::{DateTime, Local};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
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
      root: Path::new(cfg.folder.as_str())
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string(),
      listener: Arc::new(Mutex::new(listener)),
      user_map: Arc::new(Mutex::new(HashMap::new())),
      data_map: Arc::new(Mutex::new(HashMap::new())),
    })
  }

  #[allow(unused_must_use)]
  pub async fn listen(&self) {
    println!("Listening on {}:{}", self.host, self.port);
    println!("Root folder: {}", self.root);
    // todo: token pool and idle pool
    loop {
      if let Ok((socket, addr)) = self.listener.lock().await.accept().await {
        // let data_map = self.data_map.clone();
        let shared_self = self.clone();
        tokio::spawn(async move {
          shared_self.handle(socket, addr).await;
        });
      } else {
        continue;
      }
    }
  }

  pub async fn handle(&self, mut socket: TcpStream, addr: SocketAddr) {
    let user_map = self.user_map.clone();
    if !user_map.lock().await.contains_key(&addr) {
      println!("New user: {}", addr);
      socket
        .write_all(b"220 rftp.whiteffire.cn FTP server ready.\r\n")
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
      let req = String::from_utf8_lossy(&buf[..n]).to_string();
      let resp = self.dispatch(req, addr).await.unwrap();
      socket.write_all(resp.as_bytes()).await.unwrap();
      // socket.flush().await.unwrap();
    }
  }

  async fn dispatch(&self, req: String, addr: SocketAddr) -> Result<String, io::Error> {
    let cmd = message::parse_command(req);
    println!("Addr: {}, Cmd: {:?}", addr, cmd);
    let user_map = self.user_map.clone();
    let mut user_map = user_map.lock().await;
    let current_user = user_map.get_mut(&addr).unwrap();
    match cmd {
      message::FtpCommand::USER(username) => {
        current_user.username = username;
        current_user.status = user::UserStatus::Logging;
        Ok("331 User name okay, need password.\r\n".to_string())
      }
      message::FtpCommand::PASS(_) => {
        if current_user.username == "anonymous" {
          current_user.status = user::UserStatus::Active;
          Ok("230 User logged in, proceed.\r\n".to_string())
        } else {
          Ok("530 Not logged in.\r\n".to_string())
        }
      }
      message::FtpCommand::PORT(addr) => {
        let stream = TcpStream::connect(addr).await.unwrap();
        current_user.sessions.insert(
          addr,
          TransferSession {
            mode: user::TransferMode::Port(Mutex::new(stream)),
            total_size: 0,
            finished_size: 0,
            file_name: String::new(),
          },
        );
        Ok("200 PORT command successful.\r\n".to_string())
      }
      message::FtpCommand::PASV => {
        let listener = self.generate_pasv_addr().await.unwrap();
        let addr_str = listener
          .local_addr()
          .unwrap_or(SocketAddr::from_str(format!("{}:{}", self.host, self.port).as_str()).unwrap())
          .ip()
          .to_string()
          .replace(".", ",");
        let port = addr.port();
        current_user.sessions.insert(
          listener.local_addr().unwrap(),
          TransferSession {
            mode: user::TransferMode::Passive(listener),
            total_size: 0,
            finished_size: 0,
            file_name: String::new(),
          },
        );
        Ok(format!(
          "227 Entering Passive Mode ({},{},{})\r\n",
          addr_str,
          port / 256,
          port % 256,
        ))
      }
      message::FtpCommand::RETR(file_name) => {
        if let Some(session) = current_user.sessions.get_mut(&addr) {
          session.file_name = file_name.clone();
          // Path join self.root, current_user.pwd, file_name
          let path = Path::new(&self.root)
            .join(&current_user.pwd)
            .join(file_name);
          if !Path::exists(&path) {
            return Ok("550 File not found.\r\n".to_string());
          }
          match session.mode.borrow() {
            user::TransferMode::Port(stream) => {
              let file = fs::read(path).unwrap();
              stream.lock().await.write_all(&file).await.unwrap();
            }
            user::TransferMode::Passive(listener) => {
              let (mut stream, _) = listener.accept().await.unwrap();
              let file = fs::read(path).unwrap();
              stream.write_all(&file).await.unwrap();
            }
          }
        } else {
          return Ok("425 Can't open data connection.\r\n".to_string());
        }
        Ok("150 File status okay; about to open data connection.\r\n".to_string())
      }
      message::FtpCommand::STOR(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("150 File status okay; about to open data connection.\r\n".to_string())
      }
      message::FtpCommand::ABOR => {
        current_user.status = user::UserStatus::Active;
        Ok("200 ABOR command successful.\r\n".to_string())
      }
      message::FtpCommand::QUIT => {
        user_map.remove(&addr);
        Ok("221 Goodbye.\r\n".to_string())
      }
      message::FtpCommand::SYST => Ok("215 UNIX Type: L8\r\n".to_string()),
      message::FtpCommand::TYPE => Ok("200 Type set to I\r\n.".to_string()),
      message::FtpCommand::RNFR(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("350 Requested file action pending further information.\r\n".to_string())
      }
      message::FtpCommand::RNTO(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("250 Requested file action okay, completed.\r\n".to_string())
      }
      message::FtpCommand::PWD => Ok(format!(
        "257 \"{}\" is the current directory.\r\n",
        current_user.pwd
      )),
      message::FtpCommand::CWD(dir) => {
        current_user.pwd = dir;
        Ok("250 Requested file action okay, completed.\r\n".to_string())
      }
      message::FtpCommand::MKD(dir) => Ok("257 Directory created.\r\n".to_string()),
      message::FtpCommand::RMD(dir) => {
        Ok("250 Requested file action okay, completed.\r\n".to_string())
      }
      message::FtpCommand::LIST(optional_dir) => {
        let path = match optional_dir {
          Some(path) => Path::new(&self.root).join(&current_user.pwd).join(path),
          None => Path::new(&self.root).join(&current_user.pwd),
        };
        let path = path.canonicalize().unwrap();
        if !path.starts_with(&self.root) {
          return Ok("550 Permission denied.\r\n".to_string());
        }
        let mut files = fs::read_dir(path).unwrap();
        let mut list = String::new();
        // https://files.stairways.com/other/ftp-list-specs-info.txt
        // http://cr.yp.to/ftp/list/binls.html
        while let Some(file) = files.next() {
          let file = file.unwrap();
          let metadata = file.metadata().unwrap();
          let file_name = file.file_name().into_string().unwrap();
          let file_type = if metadata.is_dir() { "d" } else { "-" };
          let file_size = metadata.len();
          let file_size = format!("{:>13}", file_size);
          let file_time = file
            .metadata()
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap();
          let file_time = DateTime::from_timestamp(file_time.as_secs() as i64, 0)
            .unwrap()
            .with_timezone(&Local)
            .format("%b %d %H:%M")
            .to_string();
          let permission = if metadata.is_dir() {
            "rwxr-xr-x"
          } else {
            "rw-r--r--"
          };
          list.push_str(&format!(
            "{}{} 1 owner group {} {} {}\n",
            file_type, permission, file_size, file_time, file_name
          ));
        }
        println!("List: \n{}", list);
        Ok(format!(
          "150 Opening ASCII mode data connection for file list\n{}226 Transfer complete\r\n",
          list
        ))
      }
      message::FtpCommand::REST => {
        Ok("350 Requested file action pending further information.\r\n".to_string())
      }
      message::FtpCommand::DELE(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("250 Requested file action okay, completed.\r\n".to_string())
      }
      message::FtpCommand::STAT => Ok("211 End.\r\n".to_string()),
      message::FtpCommand::STOU => {
        Ok("202 Command not implemented, superfluous at this site.\r\n".to_string())
      }
      message::FtpCommand::APPE(file_name) => {
        let session = current_user.sessions.get_mut(&addr).unwrap();
        session.file_name = file_name;
        Ok("150 File status okay; about to open data connection.\r\n".to_string())
      }
      message::FtpCommand::ALLO(_) => Ok("200 Command okay.\r\n".to_string()),
      message::FtpCommand::NOOP => Ok("200 NOOP ok.\r\n".to_string()),
      message::FtpCommand::FEAT => Ok("211-Features:\nPASV\n211 End.\r\n".to_string()),
    }
  }

  async fn generate_pasv_addr(&self) -> Option<TcpListener> {
    for port in 49152..65535 {
      let addr = format!("{}:{}", self.host, port);
      if let Ok(addr) = addr.parse::<SocketAddr>() {
        match TcpListener::bind(addr).await {
          Ok(listener) => return Some(listener),
          Err(_) => continue,
        }
      }
    }
    None
  }
}
