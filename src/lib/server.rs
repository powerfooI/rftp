use crate::arg_parser::Args;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::lib::commands::{parse_command, FtpCommand};
use crate::lib::ftp::FtpServer;
use crate::lib::user::User;

#[derive(Debug, Clone)]
pub struct Server {
  pub host: String,
  pub port: u16,
  pub root: String,
  pub listener: Arc<TcpListener>,
  pub user_map: Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<User>>>>>,
}

impl Server {
  pub async fn new(cfg: Args) -> Result<Self, tokio::io::Error> {
    let listener = TcpListener::bind(format!("{}:{}", cfg.host, cfg.port)).await?;

    Ok(Self {
      host: cfg.host,
      port: cfg.port,
      root: Path::new(cfg.folder.as_str())
        .canonicalize()?
        .to_str()
        .ok_or(io::Error::new(
          io::ErrorKind::NotFound,
          "Failed to get root path",
        ))?
        .to_string(),
      listener: Arc::new(listener),
      user_map: Arc::new(Mutex::new(HashMap::new())),
    })
  }

  pub async fn listen(&self) {
    println!("Listening on {}:{}", self.host, self.port);
    println!("Root folder: {}", self.root);
    loop {
      if let Ok((socket, addr)) = self.listener.accept().await {
        let shared_self = self.clone();
        tokio::spawn(async move {
          shared_self.handle(socket, addr).await;
        });
      } else {
        continue;
      }
    }
  }

  pub async fn handle(&self, socket: TcpStream, addr: SocketAddr) {
    let user_map = self.user_map.clone();
    let (mut reader, mut writer) = socket.into_split();

    println!("New connection: {}", addr);
    {
      let mut user_map_locked = user_map.lock().await;
      if !user_map_locked.contains_key(&addr) {
        if let Err(e) = writer
          .write_all(b"220 rftp.whiteffire.cn FTP server ready.\r\n")
          .await
        {
          println!("Failed to send welcome message: {}", e);
          return;
        }

        let new_user = match User::new_anonymous(addr, &self.root) {
          Ok(u) => u,
          Err(e) => {
            println!("Failed to create new user: {}", e);
            return;
          }
        };

        user_map_locked.insert(addr.clone(), Arc::new(Mutex::new(new_user)));
      }
    }
    let writer_guard = Arc::new(Mutex::new(writer));
    loop {
      let mut buf = vec![0; 2048];
      let req = {
        let n = match reader.read(&mut buf).await {
          Ok(n) => n,
          Err(_) => {
            println!("Connection closed: {}", addr);
            user_map.lock().await.remove(&addr);
            return;
          }
        };
        String::from_utf8_lossy(&buf[..n]).to_string()
      };

      if req.is_empty() {
        continue;
      }
      let cloned_writer = writer_guard.clone();
      let user = match user_map.lock().await.get(&addr) {
        Some(u) => u.clone(),
        None => {
          println!("User not found: {}", addr);
          return;
        }
      };
      let cloned_self = self.clone();

      let cmd = parse_command(req);
      println!("Addr: {}, Cmd: {:?}", addr, cmd);

      if cmd == FtpCommand::QUIT {
        {
          let _ = self.quit(cloned_writer, user).await;
        }
        user_map.lock().await.remove(&addr);
        return;
      }

      tokio::spawn(async move {
        let cloned = cloned_writer.clone();
        let error_msg = match cloned_self.dispatch(cloned_writer.clone(), cmd, user).await {
          Err(e) => String::from(e.to_string()),
          Ok(_) => String::new(),
        };
        if !error_msg.is_empty() {
          println!("Error occurs: {}", error_msg);
          let mut writer = cloned.lock().await;
          if let Err(e) = writer
            .write_all(format!("550 Error occurs: {}", error_msg).as_bytes())
            .await
          {
            println!("Failed to respond error: {}", e)
          }
        }
      });
    }
  }

  async fn dispatch(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    cmd: FtpCommand,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    match cmd {
      FtpCommand::USER(username) => self.user(control, user, username).await,
      FtpCommand::PASS(pwd) => self.pass(control, user, pwd).await,
      FtpCommand::PORT(addr) => self.port_mode(control, user, addr).await,
      FtpCommand::PASV => self.passive_mode(control, user).await,
      FtpCommand::RETR(file_name) => self.retrieve(control, user, file_name).await,
      FtpCommand::STOR(file_name) => self.store(control, user, file_name).await,
      FtpCommand::ABOR => self.abort(control, user).await,
      FtpCommand::QUIT => {
        // NOTES: QUIT command is handled in the main loop
        self.quit(control, user).await
      }
      FtpCommand::SYST => self.system_info(control, user).await,
      FtpCommand::TYPE(type_) => self.set_type(control, user, type_).await,
      FtpCommand::RNFR(file_name) => self.rename_from(control, user, file_name).await,
      FtpCommand::RNTO(file_name) => self.rename_to(control, user, file_name).await,
      FtpCommand::PWD => self.pwd(control, user).await,
      FtpCommand::CWD(dir_name) => self.cwd(control, user, dir_name).await,
      FtpCommand::MKD(dir_name) => self.make_dir(control, user, dir_name).await,
      FtpCommand::RMD(dir_name) => self.remove_dir(control, user, dir_name).await,
      FtpCommand::LIST(optional_dir) => self.list(control, user, optional_dir).await,
      FtpCommand::REST(offset) => self.restart(control, user, offset).await,
      FtpCommand::DELE(file_name) => self.delete(control, user, file_name).await,
      FtpCommand::STAT(optional_path) => self.status(control, user, optional_path).await,
      FtpCommand::STOU => self.store_unique(control, user).await,
      FtpCommand::APPE(file_name) => self.append(control, user, file_name).await,
      FtpCommand::ALLO(size) => self.allocate(control, user, size).await,
      FtpCommand::NOOP => self.noop(control, user).await,
      FtpCommand::FEAT => self.feat(control, user).await,
      FtpCommand::CDUP => self.cd_up(control, user).await,
      FtpCommand::MDTM(filename) => self.get_modify_timestamp(control, user, filename).await,
      FtpCommand::NLST(optional_dir) => self.name_list(control, user, optional_dir).await,
    }
  }

  pub async fn generate_pasv_addr(&self) -> Result<TcpListener, Box<dyn Error>> {
    for port in 49152..65535 {
      let addr = format!("{}:{}", self.host, port);
      if let Ok(addr) = addr.parse::<SocketAddr>() {
        match TcpListener::bind(addr).await {
          Ok(listener) => return Ok(listener),
          Err(_) => continue,
        }
      }
    }
    Err("Failed to generate PASV address".into())
  }
}
