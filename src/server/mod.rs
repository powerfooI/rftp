use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub mod commands;
pub mod ftp;
pub mod user;

use commands::FtpCommand;
use ftp::FtpServer;
use user::{TransferSession, User};

use crate::arg_parser::Args;

#[derive(Debug, Clone)]
pub struct Server {
  pub host: String,
  pub port: u16,
  pub root: String,
  pub listener: Arc<Mutex<TcpListener>>,
  pub user_map: Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<User>>>>>,
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
      user_map.lock().await.insert(
        addr.clone(),
        Arc::new(Mutex::new(User::new_anonymous(addr))),
      );
    }
    let guard = Arc::new(Mutex::new(socket));
    loop {
      let mut buf = vec![0; 2048];
      let cloned = guard.clone();
      let req = {
        let mut stream = cloned.lock().await;
        let n = stream.read(&mut buf).await.unwrap();
        if n == 0 {
          return;
        }
        String::from_utf8_lossy(&buf[..n]).to_string()
      };

      self.dispatch(cloned, req, addr).await;
      // socket.flush().await.unwrap();
    }
  }

  async fn dispatch(&self, control: Arc<Mutex<TcpStream>>, req: String, addr: SocketAddr) {
    let cmd = commands::parse_command(req);
    println!("Addr: {}, Cmd: {:?}", addr, cmd);

    let user_map = self.user_map.clone();
    let mut user_map = user_map.lock().await;
    let current_user = user_map.get_mut(&addr).unwrap();
    let cloned_user = current_user.clone();
    let mut locking_user = current_user.lock().await;
    let locking_user = locking_user.borrow_mut();

    let mut control_stream = control.lock().await;
    let control_stream = control_stream.borrow_mut();

    match cmd {
      FtpCommand::USER(username) => {
        self.user(control_stream, locking_user, username).await;
      }
      FtpCommand::PASS(pwd) => {
        self.pass(control_stream, locking_user, pwd).await;
      }
      FtpCommand::PORT(addr) => {
        self.port_mode(control_stream, locking_user, addr).await;
      }
      FtpCommand::PASV => {
        self.passive_mode(control_stream, cloned_user).await;
      }
      FtpCommand::RETR(file_name) => {
        self.retrieve(control_stream, locking_user, file_name).await;
      }
      FtpCommand::STOR(file_name) => {
        self.store(control_stream, locking_user, file_name).await;
      }
      FtpCommand::ABOR => {
        self.abort(control_stream, locking_user).await;
      }
      FtpCommand::QUIT => {
        self.quit(control_stream, locking_user).await;
      }
      FtpCommand::SYST => {
        self.system_info(control_stream, locking_user).await;
      }
      FtpCommand::TYPE(type_) => {
        self.set_type(control_stream, locking_user, type_).await;
      }
      FtpCommand::RNFR(file_name) => {
        self
          .rename_from(control_stream, locking_user, file_name)
          .await;
      }
      FtpCommand::RNTO(file_name) => {
        self
          .rename_to(control_stream, locking_user, file_name)
          .await;
      }
      FtpCommand::PWD => {
        self.pwd(control_stream, locking_user).await;
      }
      FtpCommand::CWD(dir_name) => {
        self.cwd(control_stream, locking_user, dir_name).await;
      }
      FtpCommand::MKD(dir_name) => {
        self.make_dir(control_stream, locking_user, dir_name).await;
      }
      FtpCommand::RMD(dir_name) => {
        self
          .remove_dir(control_stream, locking_user, dir_name)
          .await;
      }
      FtpCommand::LIST(optional_dir) => {
        self.list(control_stream, locking_user, optional_dir).await;
      }
      FtpCommand::REST => {
        self.restart(control_stream, locking_user).await;
      }
      FtpCommand::DELE(file_name) => {
        self.delete(control_stream, locking_user, file_name).await;
      }
      FtpCommand::STAT => {
        self.status(control_stream, locking_user).await;
      }
      FtpCommand::STOU => {
        self.store_unique(control_stream, locking_user).await;
      }
      FtpCommand::APPE(file_name) => {
        self.append(control_stream, locking_user, file_name).await;
      }
      FtpCommand::ALLO(size) => {
        self.allocate(control_stream, locking_user, size).await;
      }
      FtpCommand::NOOP => {
        self.noop(control_stream, locking_user).await;
      }
      FtpCommand::FEAT => {
        self.feat(control_stream, locking_user).await;
      }
      FtpCommand::CDUP => {
        self.cd_up(control_stream, locking_user).await;
      }
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
