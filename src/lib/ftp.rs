use chrono::{DateTime, Local};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::{net::SocketAddr, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use async_trait::async_trait;

use self::user::{TransferSession, User, UserStatus};
use crate::lib::{server::Server, user};

#[async_trait]
pub trait FtpServer {
  async fn list(&self, control: &mut TcpStream, user: &mut User, optional_dir: Option<String>);
  async fn retrieve(&self, control: &mut TcpStream, user: &mut User, file_name: String);
  async fn store(&self, control: &mut TcpStream, user: &mut User, file_name: String);
  async fn make_dir(&self, control: &mut TcpStream, user: &mut User, dir_name: String);
  async fn remove_dir(&self, control: &mut TcpStream, user: &mut User, dir_name: String);
  async fn delete(&self, control: &mut TcpStream, user: &mut User, file_name: String);
  async fn cwd(&self, control: &mut TcpStream, user: &mut User, dir_name: String);
  async fn pwd(&self, control: &mut TcpStream, user: &mut User);
  async fn set_type(&self, control: &mut TcpStream, user: &mut User, type_: String);
  async fn passive_mode(&self, control: &mut TcpStream, user: &mut User, user_ref: Arc<Mutex<User>>);
  async fn port_mode(&self, control: &mut TcpStream, user: &mut User, port_addr: SocketAddr);
  async fn quit(&self, control: &mut TcpStream, user: &mut User);
  async fn noop(&self, control: &mut TcpStream, user: &mut User);
  async fn user(&self, control: &mut TcpStream, user: &mut User, username: String);
  async fn pass(&self, control: &mut TcpStream, user: &mut User, password: String);

  async fn abort(&self, control: &mut TcpStream, user: &mut User);
  async fn system_info(&self, control: &mut TcpStream, user: &mut User);
  async fn rename_from(&self, control: &mut TcpStream, user: &mut User, file_name: String);
  async fn rename_to(&self, control: &mut TcpStream, user: &mut User, file_name: String);
  async fn restart(&self, control: &mut TcpStream, user: &mut User);
  async fn status(&self, control: &mut TcpStream, user: &mut User);
  async fn store_unique(&self, control: &mut TcpStream, user: &mut User);
  async fn append(&self, control: &mut TcpStream, user: &mut User, file_name: String);
  async fn allocate(&self, control: &mut TcpStream, user: &mut User, size: u64);
  async fn feat(&self, control: &mut TcpStream, user: &mut User);
  async fn cd_up(&self, control: &mut TcpStream, user: &mut User);
}

#[async_trait]
impl FtpServer for Server {
  async fn list(&self, control: &mut TcpStream, user: &mut User, optional_dir: Option<String>) {
    let addr = user.addr.clone();
    let path = match optional_dir {
      Some(path) => Path::new(&self.root).join(&user.pwd).join(path),
      None => Path::new(&self.root).join(&user.pwd),
    };
    let path = path.canonicalize().unwrap();
    if !path.starts_with(&self.root) {
      control
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
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
        "{}{} 1 owner group {} {} {}\r\n",
        file_type, permission, file_size, file_time, file_name
      ));
    }
    let session = user.sessions.get(&addr).unwrap();
    let mut data_stream = match &session.mode {
      user::TransferMode::Port(stream) => stream.lock().await,
      user::TransferMode::Passive(stream) => stream.lock().await,
    };

    control
      .write_all(b"150 Opening ASCII mode data connection for file list\r\n")
      .await
      .unwrap();
    data_stream.write_all(list.as_bytes()).await.unwrap();
    data_stream.shutdown().await.unwrap();
    control
      .write_all(b"226 Transfer complete.\r\n")
      .await
      .unwrap();
  }

  async fn retrieve(&self, control: &mut TcpStream, user: &mut User, file_name: String) {
    let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
    let addr = user.addr;

    let session = user.sessions.get_mut(&addr).unwrap();
    session.file_name = file_name.clone();

    // Path join self.root, current_user.pwd, file_name
    if !Path::exists(&path) {
      control.write_all(b"550 File not found.\r\n").await.unwrap();
    }
    let mut data_stream = match &session.mode {
      user::TransferMode::Port(stream) => stream.lock().await,
      user::TransferMode::Passive(stream) => stream.lock().await,
    };

    let file = fs::read(path).unwrap();
    control
      .write_all(
        format!(
          "150 Opening BINARY mode data connection for {}.\r\n",
          file_name
        )
        .as_bytes(),
      )
      .await
      .unwrap();
    data_stream.write_all(&file).await.unwrap();
    data_stream.shutdown().await.unwrap();
    control
      .write_all(b"226 Transfer complete.\r\n")
      .await
      .unwrap();
  }

  async fn store(&self, control: &mut TcpStream, user: &mut User, file_name: String) {
    let addr = user.addr;
    let session = user.sessions.get_mut(&addr).unwrap();
    session.file_name = file_name.clone();
    let mut data_stream = match &session.mode {
      user::TransferMode::Port(stream) => stream.lock().await,
      user::TransferMode::Passive(stream) => stream.lock().await,
    };
    let target_path = Path::new(&self.root).join(&user.pwd).join(&file_name);
    let mut file = fs::File::create(target_path).unwrap();
    control
      .write_all(
        format!(
          "150 Opening BINARY mode data connection for {}.\r\n",
          file_name
        )
        .as_bytes(),
      )
      .await
      .unwrap();
    loop {
      let mut buf = vec![0; 1024];
      let n = data_stream.read(&mut buf).await.unwrap();
      if n == 0 {
        break;
      }
      file.write_all(&buf[..n]).unwrap();
    }

    control
      .write_all(b"226 Transfer complete.\r\n")
      .await
      .unwrap();
  }

  async fn make_dir(&self, control: &mut TcpStream, user: &mut User, dir_name: String) {
    // let parts = current_user.pwd.split("/").collect();
    match fs::create_dir(Path::new(&self.root).join(&user.pwd).join(&dir_name)) {
      Ok(_) => {
        control
          .write_all(b"257 Directory created.\r\n")
          .await
          .unwrap();
      }
      Err(e) => {
        println!("Error: {:?}", e);
        control
          .write_all(b"550 Permission denied.\r\n")
          .await
          .unwrap();
      }
    }
  }
  async fn remove_dir(&self, control: &mut TcpStream, user: &mut User, dir_name: String) {
    println!("Remove dir: {:?}", dir_name);
    println!("User pwd: {:?}", user.pwd);
    match Path::new(&self.root)
    .join(&user.pwd)
    .join(&dir_name)
    .canonicalize() {
        Ok(new_path) => {
          {
            if !new_path.starts_with(&self.root) {
              control
                .write_all(b"550 Permission denied.\r\n")
                .await
                .unwrap();
            }
            if !new_path.exists() {
              control
                .write_all(b"553 Not found.\r\n")
                .await
                .unwrap();
            }
            if let Ok(_) = fs::remove_dir(new_path) {
              control
                .write_all(b"200 Remove completed.\r\n")
                .await
                .unwrap();
            } else {
              control
                .write_all(b"550 Failed to remove directory.\r\n")
                .await
                .unwrap();
            }
          } 
        }
        Err(e) => {
          println!("Error: {:?}", e);
          control
            .write_all(b"550 Path error.\r\n")
            .await
            .unwrap();
        }
    }
  }
  async fn delete(&self, control: &mut TcpStream, user: &mut User, file_name: String) {
    let addr = user.addr;
    let session = user.sessions.get_mut(&addr).unwrap();
    session.file_name = file_name;
    control
      .write_all(b"250 Requested file action okay, completed.\r\n")
      .await
      .unwrap();
  }
  async fn cwd(&self, control: &mut TcpStream, user: &mut User, dir_name: String) {
    if let Ok(new_path) = Path::new(&self.root)
      .join(&user.pwd)
      .join(&dir_name)
      .canonicalize()
    {
      if !new_path.starts_with(&self.root) {
        control
          .write_all(b"550 Permission denied.\r\n")
          .await
          .unwrap();
      }
      if !new_path.starts_with(&self.root) {
        control
          .write_all(b"550 Permission denied.\r\n")
          .await
          .unwrap();
      }
      println!("new path: {:?}", Path::new(&user.pwd).join(&dir_name));
      user.pwd = new_path
        .to_str()
        .unwrap()
        .to_string()
        .replace(&self.root, ".");
      control
        .write_all(b"250 Requested file action okay, completed.\r\n")
        .await
        .unwrap();
    } else {
      return control
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
    }
  }
  async fn pwd(&self, control: &mut TcpStream, user: &mut User) {
    control
      .write_all(format!("257 \"{}\" is the current directory.\r\n", user.pwd).as_bytes())
      .await
      .unwrap();
  }
  async fn set_type(&self, control: &mut TcpStream, _: &mut User, _: String) {
    control.write_all(b"200 Type set to I.\r\n").await.unwrap()
  }
  async fn passive_mode(&self, control: &mut TcpStream, user: &mut User, user_ref: Arc<Mutex<User>>) {
    let cloned = user_ref.clone();
    let addr = user.addr;
    let listener = self.generate_pasv_addr().await.unwrap();
    let listen_addr = listener
      .local_addr()
      .unwrap_or(SocketAddr::from_str(format!("{}:{}", self.host, self.port).as_str()).unwrap());
    let ip = listen_addr.ip().to_string().replace(".", ",");
    let port = listen_addr.port();

    control
      .write_all(
        format!(
          "227 Entering Passive Mode ({},{},{})\r\n",
          ip,
          port / 256,
          port % 256,
        )
        .as_bytes(),
      )
      .await
      .unwrap();

    tokio::spawn(async move {
      let (stream, _) = listener.accept().await.unwrap();
      cloned.lock().await.sessions.insert(
        addr,
        TransferSession {
          mode: user::TransferMode::Passive(Mutex::new(stream)),
          total_size: 0,
          finished_size: 0,
          file_name: String::new(),
        },
      );
      println!("Passive connection established.")
    });
  }
  async fn port_mode(&self, control: &mut TcpStream, user: &mut User, port_addr: SocketAddr) {
    let stream = TcpStream::connect(port_addr).await.unwrap();
    user.sessions.insert(
      user.addr,
      TransferSession {
        mode: user::TransferMode::Port(Mutex::new(stream)),
        total_size: 0,
        finished_size: 0,
        file_name: String::new(),
      },
    );
    control
      .write_all(b"200 PORT command successful.\r\n")
      .await
      .unwrap();
  }
  async fn quit(&self, control: &mut TcpStream, user: &mut User) {
    let addr = user.addr;
    user.sessions.remove(&addr);
    control.write_all(b"221 Goodbye.\r\n").await.unwrap();
  }
  async fn noop(&self, control: &mut TcpStream, user: &mut User) {
    control.write_all(b"200 NOOP ok.\r\n").await.unwrap();
  }

  async fn user(&self, control: &mut TcpStream, user: &mut User, username: String) {
    user.username = username;
    user.status = UserStatus::Logging;
    control
      .write_all(b"331 User name okay, need password.\r\n")
      .await
      .unwrap();
  }

  async fn pass(&self, control: &mut TcpStream, user: &mut User, _: String) {
    user.status = UserStatus::Active;
    control
      .write_all(b"230 User logged in, proceed.\r\n")
      .await
      .unwrap();
  }
  async fn abort(&self, control: &mut TcpStream, user: &mut User) {
    user.status = UserStatus::Active;
    control
      .write_all(b"200 ABOR command successful.\r\n")
      .await
      .unwrap();
  }
  async fn system_info(&self, control: &mut TcpStream, user: &mut User) {
    control.write_all(b"215 UNIX Type: L8\r\n").await.unwrap()
  }
  async fn rename_from(&self, control: &mut TcpStream, user: &mut User, file_name: String) {
    let addr = user.addr;
    let session = user.sessions.get_mut(&addr).unwrap();
    session.file_name = file_name;
    control
      .write_all(b"350 Requested file action pending further information.\r\n")
      .await
      .unwrap();
  }
  async fn rename_to(&self, control: &mut TcpStream, user: &mut User, file_name: String) {
    let addr = user.addr;
    let session = user.sessions.get_mut(&addr).unwrap();
    session.file_name = file_name;
    control
      .write_all(b"250 Requested file action okay, completed.\r\n")
      .await
      .unwrap();
  }
  async fn restart(&self, control: &mut TcpStream, user: &mut User) {
    control
      .write_all(b"350 Requested file action pending further information.\r\n")
      .await
      .unwrap();
  }
  async fn status(&self, control: &mut TcpStream, user: &mut User) {
    control.write_all(b"211 End.\r\n").await.unwrap();
  }
  async fn store_unique(&self, control: &mut TcpStream, user: &mut User) {
    control
      .write_all(b"202 Command not implemented, superfluous at this site.\r\n")
      .await
      .unwrap();
  }
  async fn append(&self, control: &mut TcpStream, user: &mut User, file_name: String) {
    let addr = user.addr;
    let session = user.sessions.get_mut(&addr).unwrap();
    session.file_name = file_name;
    control
      .write_all(b"150 File status okay; about to open data connection.\r\n")
      .await
      .unwrap();
  }
  async fn allocate(&self, control: &mut TcpStream, user: &mut User, size: u64) {
    control.write_all(b"200 Command okay.\r\n").await.unwrap();
  }
  async fn feat(&self, control: &mut TcpStream, user: &mut User) {
    control
      .write_all(b"502 Command not implemented.\r\n")
      .await
      .unwrap();
  }
  async fn cd_up(&self, control: &mut TcpStream, user: &mut User) {
    let mut new_path = Path::new(&self.root).join(&user.pwd);
    if new_path.pop() {
      let new_path = new_path.canonicalize().unwrap();
      if new_path.starts_with(&self.root) {
        user.pwd = new_path
          .to_str()
          .unwrap()
          .to_string()
          .replace(&self.root, ".");
      } else {
        control
          .write_all(b"550 Permission denied.\r\n")
          .await
          .unwrap();
      }
    } else {
      control
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
    }
    control
      .write_all(b"250 Directory successfully changed.\r\n")
      .await
      .unwrap();
  }
}
