use chrono::{DateTime, Local};
use std::error::Error;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{net::SocketAddr, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use uuid::Uuid;

use async_trait::async_trait;

use crate::lib::server::Server;
use crate::lib::session::*;
use crate::lib::user::*;

#[async_trait]
pub trait FtpServer {
  async fn list(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
  );
  async fn retrieve(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  );
  async fn store(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  );
  async fn make_dir(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  );
  async fn remove_dir(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  );
  async fn delete(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  );
  async fn cwd(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  );
  async fn pwd(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn set_type(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    type_: String,
  );
  async fn passive_mode(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn port_mode(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    port_addr: SocketAddr,
  );
  async fn quit(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn noop(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn user(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    username: String,
  );
  async fn pass(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    password: String,
  );

  async fn abort(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn system_info(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn rename_from(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  );
  async fn rename_to(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  );
  async fn restart(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>, offset: u64);
  async fn status(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_path: Option<String>,
  );
  async fn store_unique(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn append(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  );
  async fn allocate(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>, size: u64);
  async fn feat(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn cd_up(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>);
  async fn get_modify_timestamp(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  );
  async fn name_list(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
  );
}

#[async_trait]
trait FtpHelper {
  async fn list_files(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
    name_only: bool,
  );

  async fn store_file(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  );
}

#[async_trait]
impl FtpHelper for Server {
  async fn list_files(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
    name_only: bool,
  ) {
    let mut control = control.lock().await;
    let user = user.lock().await;
    let path = match optional_dir {
      Some(path) => Path::new(&self.root).join(&user.pwd).join(path),
      None => Path::new(&self.root).join(&user.pwd),
    };
    if !path.exists() {
      control
        .write_all(b"550 No such file or directory.\r\n")
        .await
        .unwrap();
      return;
    }
    let path = path.canonicalize().unwrap();
    if !path.starts_with(&self.root) {
      control
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
      return;
    }

    let list =
      get_list_lines(&path, name_only).unwrap_or_else(|_| "Something wrong.\r\n".to_string());

    let session = user.session.clone().unwrap();
    let mut session = session.lock().await;
    let data_stream = session.get_stream();
    let mut data_stream = data_stream.lock().await;

    control
      .write_all(b"150 Opening ASCII mode data connection for file list\r\n")
      .await
      .unwrap();
    data_stream.write_all(list.as_bytes()).await.unwrap();
    data_stream.shutdown().await.unwrap();
    session.set_finished(true);
    control
      .write_all(b"226 Transfer complete.\r\n")
      .await
      .unwrap();
  }

  async fn store_file(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) {
    let (target_path, mut offset) = {
      let user = user.lock().await;
      let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
      let session = user.get_session().unwrap();
      let mut session = session.lock().await;
      session.file_name = file_name.clone();

      (path, session.offset)
    };

    if !target_path.starts_with(&self.root) {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
      return;
    }

    {
      control
        .lock()
        .await
        .write_all(
          format!(
            "150 Opening BINARY mode data connection for {}.\r\n",
            file_name
          )
          .as_bytes(),
        )
        .await
        .unwrap();
    }

    let mut file = if target_path.exists() {
      let meta = target_path.metadata().unwrap();
      if meta.is_dir() {
        control
          .lock()
          .await
          .write_all(b"550 Permission denied, the path is a directory.\r\n")
          .await
          .unwrap();
        return;
      }
      if offset == 0 {
        control
          .lock()
          .await
          .write_all(b"550 Permission denied, the file exists.\r\n")
          .await
          .unwrap();
      }
      if offset > meta.len() {
        offset = meta.len();
      }
      let mut file = fs::File::open(target_path).unwrap();
      file.seek(std::io::SeekFrom::Start(offset)).unwrap();
      file
    } else {
      fs::File::create(target_path).unwrap()
    };

    loop {
      let user = user.lock().await;
      let session = user.session.clone().unwrap();
      let mut session = session.lock().await;

      if session.aborted {
        break;
      }

      let data_stream = session.get_stream();
      let mut data_stream = data_stream.lock().await;

      let mut buf = vec![0; 1024];
      let n = data_stream.read(&mut buf).await.unwrap();

      if n == 0 {
        break;
      }
      file.write_all(&buf[..n]).unwrap();
      session.finished_size += n as u64;
    }

    let user = user.lock().await;
    let session = user.session.clone().unwrap();
    let mut session = session.lock().await;

    let data_stream = session.get_stream();
    let mut data_stream = data_stream.lock().await;
    data_stream.shutdown().await.unwrap();
    if session.aborted {
      control
        .lock()
        .await
        .write_all(b"226 Connection closed; transfer aborted.\r\n")
        .await
        .unwrap();
    } else {
      session.finished = true;
      control
        .lock()
        .await
        .write_all(b"226 Transfer complete.\r\n")
        .await
        .unwrap();
    }
  }
}

fn file_path_to_list_item(path: &PathBuf, name_only: bool) -> Result<String, Box<dyn Error>> {
  // https://files.stairways.com/other/ftp-list-specs-info.txt
  // http://cr.yp.to/ftp/list/binls.html
  let metadata = fs::metadata(&path)?;
  let file_name = path.file_name().unwrap().to_str().unwrap();
  if name_only {
    return Ok(format!("{}\r\n", file_name).to_string());
  }
  let file_size = format!("{:>13}", metadata.len());
  let file_type = if metadata.is_dir() { "d" } else { "-" };
  let file_time = metadata
    .modified()?
    .duration_since(std::time::SystemTime::UNIX_EPOCH)?;
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
  Ok(
    format!(
      "{}{} 1 owner group {} {} {}\r\n",
      file_type, permission, file_size, file_time, file_name
    )
    .to_string(),
  )
}

fn get_list_lines(path: &PathBuf, name_only: bool) -> Result<String, Box<dyn Error>> {
  let mut list = String::new();
  if path.is_dir() {
    let mut files = fs::read_dir(&path)?;
    while let Some(file) = files.next() {
      let file = file?;
      list.push_str(file_path_to_list_item(&file.path(), name_only)?.as_str());
    }
  } else {
    list.push_str(file_path_to_list_item(path, name_only)?.as_str());
  }
  Ok(list)
}

#[async_trait]
impl FtpServer for Server {
  async fn list(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
  ) {
    self.list_files(control, user, optional_dir, false).await;
  }

  async fn name_list(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
  ) {
    self.list_files(control, user, optional_dir, true).await;
  }

  async fn retrieve(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) {
    let (path, offset) = {
      let user = user.lock().await;

      let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
      let session = user.session.clone().unwrap();
      let mut session = session.lock().await;
      session.file_name = file_name.clone();

      (path, session.offset)
    };

    // Path join self.root, current_user.pwd, file_name
    if !Path::exists(&path) {
      control
        .lock()
        .await
        .write_all(b"550 File not found.\r\n")
        .await
        .unwrap();
    }

    {
      control
        .lock()
        .await
        .write_all(
          format!(
            "150 Opening BINARY mode data connection for {}.\r\n",
            file_name
          )
          .as_bytes(),
        )
        .await
        .unwrap();
    }

    let mut file = fs::File::open(path).unwrap();
    if offset > 0 {
      let meta = file.metadata().unwrap();
      let file_size = meta.len();
      if offset >= file_size {
        control
          .lock()
          .await
          .write_all(b"550 Offset out of range.\r\n")
          .await
          .unwrap();
        return;
      }
      file.seek(std::io::SeekFrom::Start(offset)).unwrap();
    }
    loop {
      let user = user.lock().await;
      let session = user.session.clone().unwrap();
      let mut session = session.lock().await;
      if session.aborted {
        break;
      }
      let data_stream = session.get_stream();
      let mut data_stream = data_stream.lock().await;
      let mut buf = vec![0u8; 1024];
      let n = file.read(&mut buf).unwrap();
      if n == 0 {
        break;
      }
      data_stream.write_all(&buf[..n]).await.unwrap();
      session.finished_size += n as u64;
    }

    let user = user.lock().await;
    let session = user.session.clone().unwrap();
    let mut session = session.lock().await;

    let data_stream = session.get_stream();
    let mut data_stream = data_stream.lock().await;
    data_stream.shutdown().await.unwrap();

    if session.aborted {
      control
        .lock()
        .await
        .write_all(b"226 Connection closed; transfer aborted.\r\n")
        .await
        .unwrap();
    } else {
      session.finished = true;
      control
        .lock()
        .await
        .write_all(b"226 Transfer complete.\r\n")
        .await
        .unwrap();
    }
  }

  async fn store(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) {
    self.store_file(control, user, file_name).await;
  }

  async fn make_dir(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) {
    let user = user.lock().await;
    // let parts = current_user.pwd.split("/").collect();
    match fs::create_dir(Path::new(&self.root).join(&user.pwd).join(&dir_name)) {
      Ok(_) => {
        control
          .lock()
          .await
          .write_all(b"257 Directory created.\r\n")
          .await
          .unwrap();
      }
      Err(e) => {
        println!("Error: {:?}", e);
        control
          .lock()
          .await
          .write_all(b"550 Permission denied.\r\n")
          .await
          .unwrap();
      }
    }
  }
  async fn remove_dir(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) {
    let user = user.lock().await;
    match Path::new(&self.root)
      .join(&user.pwd)
      .join(&dir_name)
      .canonicalize()
    {
      Ok(new_path) => {
        if !new_path.starts_with(&self.root) {
          control
            .lock()
            .await
            .write_all(b"550 Permission denied.\r\n")
            .await
            .unwrap();
        }
        if !new_path.exists() {
          control
            .lock()
            .await
            .write_all(b"553 Not found.\r\n")
            .await
            .unwrap();
        }
        if let Ok(_) = fs::remove_dir(new_path) {
          control
            .lock()
            .await
            .write_all(b"200 Remove completed.\r\n")
            .await
            .unwrap();
        } else {
          control
            .lock()
            .await
            .write_all(b"550 Failed to remove directory.\r\n")
            .await
            .unwrap();
        }
      }
      Err(e) => {
        println!("Error: {:?}", e);
        control
          .lock()
          .await
          .write_all(b"550 Path error.\r\n")
          .await
          .unwrap();
      }
    }
  }
  async fn delete(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) {
    let user = user.lock().await;
    let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
    if !path.exists() {
      control
        .lock()
        .await
        .write_all(b"553 Not found.\r\n")
        .await
        .unwrap();
      return;
    }
    if !path.starts_with(&self.root) {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
      return;
    }
    match fs::remove_file(path) {
      Ok(_) => {
        control
          .lock()
          .await
          .write_all(b"250 Requested file action okay, completed.\r\n")
          .await
          .unwrap();
      }
      Err(e) => {
        println!("Error: {:?}", e);
        control
          .lock()
          .await
          .write_all(b"550 Failed to remove file.\r\n")
          .await
          .unwrap();
      }
    };
  }
  async fn cwd(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) {
    let mut user = user.lock().await;
    let dir_name = dir_name.trim_start_matches("/");
    if dir_name.is_empty() {
      user.pwd = ".".to_string();
      control
        .lock()
        .await
        .write_all(b"250 Requested file action okay, completed.\r\n")
        .await
        .unwrap();
      return;
    } else if dir_name == "." {
      control
        .lock()
        .await
        .write_all(b"250 PWD not changed.\r\n")
        .await
        .unwrap();
      return;
    }
    if let Ok(new_path) = Path::new(&self.root)
      .join(&user.pwd)
      .join(&dir_name)
      .canonicalize()
    {
      if !new_path.starts_with(&self.root) {
        control
          .lock()
          .await
          .write_all(b"550 Permission denied.\r\n")
          .await
          .unwrap();
      }
      if !new_path.starts_with(&self.root) {
        control
          .lock()
          .await
          .write_all(b"550 Permission denied.\r\n")
          .await
          .unwrap();
      }
      user.pwd = new_path
        .to_str()
        .unwrap()
        .to_string()
        .replace(&self.root, ".");
      control
        .lock()
        .await
        .write_all(b"250 Requested file action okay, completed.\r\n")
        .await
        .unwrap();
    } else {
      return control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
    }
  }
  async fn pwd(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    let user = user.lock().await;
    control
      .lock()
      .await
      .write_all(format!("257 \"{}\" is the current directory.\r\n", &user.pwd).as_bytes())
      .await
      .unwrap();
  }
  async fn set_type(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    type_: String,
  ) {
    // let session = user.sessions.get_mut(&user.addr).unwrap();
    match type_.to_uppercase().as_str() {
      "A" => {
        user.lock().await.trans_type = TransferType::ASCII;
        control
          .lock()
          .await
          .write_all(b"200 Type set to ASCII.\r\n")
          .await
          .unwrap();
      }
      "I" => {
        user.lock().await.trans_type = TransferType::Binary;
        control
          .lock()
          .await
          .write_all(b"200 Type set to Binary.\r\n")
          .await
          .unwrap();
      }
      _ => {
        control
          .lock()
          .await
          .write_all(b"504 Command not implemented for that parameter.\r\n")
          .await
          .unwrap();
      }
    }
  }
  async fn passive_mode(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    let cloned = user.clone();
    let listener = self.generate_pasv_addr().await.unwrap();
    let listen_addr = listener
      .local_addr()
      .unwrap_or(SocketAddr::from_str(format!("{}:{}", self.host, self.port).as_str()).unwrap());
    let ip = listen_addr.ip().to_string().replace(".", ",");
    let port = listen_addr.port();

    control
      .lock()
      .await
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
    // let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
      let (stream, _) = listener.accept().await.unwrap();
      cloned
        .lock()
        .await
        .set_new_session(TransferSession::new(TransferMode::Passive(Arc::new(
          Mutex::new(stream),
        ))));
    });
  }

  async fn port_mode(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    port_addr: SocketAddr,
  ) {
    let mut user = user.lock().await;
    let stream = TcpStream::connect(port_addr).await.unwrap();

    user.set_new_session(TransferSession::new(TransferMode::Port(Arc::new(
      Mutex::new(stream),
    ))));

    control
      .lock()
      .await
      .write_all(b"200 PORT command successful.\r\n")
      .await
      .unwrap();
  }

  async fn quit(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    let mut user = user.lock().await;
    user.session = None;
    let mut locking = control.lock().await;
    locking.write_all(b"221 Goodbye.\r\n").await.unwrap();
    locking.shutdown().await.unwrap();
  }

  async fn noop(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    control
      .lock()
      .await
      .write_all(b"200 NOOP ok.\r\n")
      .await
      .unwrap();
  }

  async fn user(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    username: String,
  ) {
    let mut user = user.lock().await;
    user.username = username;
    user.status = UserStatus::Logging;
    control
      .lock()
      .await
      .write_all(b"331 User name okay, need password.\r\n")
      .await
      .unwrap();
  }

  async fn pass(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>, _: String) {
    {
      user.lock().await.status = UserStatus::Active;
    }
    control
      .lock()
      .await
      .write_all(b"230 User logged in, proceed.\r\n")
      .await
      .unwrap();
  }
  async fn abort(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    let mut locking = user.lock().await;
    locking.status = UserStatus::Active;
    let session = locking.session.clone().unwrap();
    let mut session = session.lock().await;
    session.aborted = true;
    control
      .lock()
      .await
      .write_all(b"226 ABOR command processed.\r\n") // '426', '225', '226'
      .await
      .unwrap();
  }
  async fn system_info(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    control
      .lock()
      .await
      .write_all(b"215 UNIX Type: L8\r\n")
      .await
      .unwrap()
  }
  async fn rename_from(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) {
    let user = user.lock().await;
    let session = user.session.clone().unwrap();
    let mut session = session.lock().await;
    session.file_name = file_name;
    control
      .lock()
      .await
      .write_all(b"350 Requested file action pending further information.\r\n")
      .await
      .unwrap();
  }
  async fn rename_to(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) {
    let user = user.lock().await;
    let pwd = user.pwd.clone();
    let session = user.session.clone().unwrap();
    let mut session = session.lock().await;
    let old_path = Path::new(&self.root).join(&pwd).join(&session.file_name);
    let new_path = Path::new(&self.root).join(&pwd).join(&file_name);
    fs::rename(old_path, new_path).unwrap();
    session.file_name = file_name;
    {
      control
        .lock()
        .await
        .write_all(b"250 Requested file action okay, completed.\r\n")
        .await
        .unwrap();
    }
  }
  async fn restart(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    offset: u64,
  ) {
    let user = user.lock().await;
    let session = user.session.clone().unwrap();
    let mut session = session.lock().await;
    session.offset = offset;
    control
      .lock()
      .await
      .write_all(b"350 Requested file action pending further information.\r\n")
      .await
      .unwrap();
  }
  async fn status(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_path: Option<String>,
  ) {
    let user = user.lock().await;
    let mut control = control.lock().await;
    match optional_path {
      Some(path_str) => {
        let path = Path::new(&self.root).join(&user.pwd).join(&path_str);
        if !path.exists() {
          control.write_all(b"553 Not found.\r\n").await.unwrap();
        } else {
          let path = path.canonicalize().unwrap();
          if !path.starts_with(&self.root) {
            control
              .write_all(b"550 Permission denied.\r\n")
              .await
              .unwrap();
          }
          let list =
            get_list_lines(&path, false).unwrap_or_else(|_| "Something wrong.\r\n".to_string());
          control
            .write_all(format!("213-Status of {}:\r\n", path_str).as_bytes())
            .await
            .unwrap();
          control.write_all(list.as_bytes()).await.unwrap();
          control.write_all(b"213 End of status.\r\n").await.unwrap();
        }
      }
      None => {
        control
          .write_all(b"211-Status of the server:\r\n")
          .await
          .unwrap();
        let mut content = String::new();
        // content.push_str(format!("Server root: {}\r\n", self.root).as_str());
        content.push_str(format!("User: {}\r\n", user.username).as_str());
        content.push_str(format!("Current directory: {}\r\n", user.pwd).as_str());
        content.push_str(format!("TYPE: {:?}\r\n", user.trans_type).as_str());
        control.write_all(b"211 End of status.\r\n").await.unwrap();
      }
    }
  }
  async fn store_unique(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    let file_name = Uuid::new_v4().to_string();
    self.store_file(control, user, file_name).await;
  }
  async fn append(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) {
    {
      let user = user.lock().await;
      let session = user.session.clone().unwrap();
      let mut session = session.lock().await;
      session.file_name = file_name.clone();
      session.offset = u64::MAX;
    }
    self.store_file(control, user, file_name).await;
  }
  async fn allocate(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>, size: u64) {
    control
      .lock()
      .await
      .write_all(b"200 Command okay.\r\n")
      .await
      .unwrap();
  }
  async fn feat(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    control
      .lock()
      .await
      .write_all(b"502 Command not implemented.\r\n")
      .await
      .unwrap();
  }
  async fn cd_up(&self, control: Arc<Mutex<OwnedWriteHalf>>, user: Arc<Mutex<User>>) {
    let mut user = user.lock().await;
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
          .lock()
          .await
          .write_all(b"550 Permission denied.\r\n")
          .await
          .unwrap();
      }
    } else {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
    }
    control
      .lock()
      .await
      .write_all(b"250 Directory successfully changed.\r\n")
      .await
      .unwrap();
  }

  async fn get_modify_timestamp(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) {
    let user = user.lock().await;
    let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
    if !path.exists() {
      control
        .lock()
        .await
        .write_all(b"553 Not found.\r\n")
        .await
        .unwrap();
      return;
    }
    if !path.starts_with(&self.root) {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await
        .unwrap();
      return;
    }
    let metadata = fs::metadata(&path).unwrap();
    let file_time = metadata
      .modified()
      .unwrap()
      .duration_since(std::time::SystemTime::UNIX_EPOCH)
      .unwrap();
    let file_time = DateTime::from_timestamp(file_time.as_secs() as i64, 0)
      .unwrap()
      .with_timezone(&Local)
      .format("%Y%m%d%H:%M%S")
      .to_string();
    control
      .lock()
      .await
      .write_all(format!("213 {}\r\n", file_time).as_bytes())
      .await
      .unwrap();
  }
}
