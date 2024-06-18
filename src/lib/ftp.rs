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
  ) -> Result<(), Box<dyn Error>>;
  async fn retrieve(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn store(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn make_dir(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn remove_dir(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn delete(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn cwd(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn pwd(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn set_type(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    type_: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn passive_mode(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn port_mode(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    port_addr: SocketAddr,
  ) -> Result<(), Box<dyn Error>>;
  async fn quit(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn noop(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn user(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    username: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn pass(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    password: String,
  ) -> Result<(), Box<dyn Error>>;

  async fn abort(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn system_info(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn rename_from(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn rename_to(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn restart(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    offset: u64,
  ) -> Result<(), Box<dyn Error>>;
  async fn status(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_path: Option<String>,
  ) -> Result<(), Box<dyn Error>>;
  async fn store_unique(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn append(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn allocate(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    size: u64,
  ) -> Result<(), Box<dyn Error>>;
  async fn feat(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn cd_up(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>>;
  async fn get_modify_timestamp(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>>;
  async fn name_list(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
  ) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
trait FtpHelper {
  async fn list_files(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
    name_only: bool,
  ) -> Result<(), Box<dyn Error>>;

  async fn store_file(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>>;
}

#[async_trait]
impl FtpHelper for Server {
  async fn list_files(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
    name_only: bool,
  ) -> Result<(), Box<dyn Error>> {
    let mut control = control.lock().await;
    let user = user.lock().await;
    let path = match optional_dir {
      Some(path) => Path::new(&self.root).join(&user.pwd).join(path),
      None => Path::new(&self.root).join(&user.pwd),
    };
    if !path.exists() {
      control
        .write_all(b"550 No such file or directory.\r\n")
        .await?;
      return Ok(());
    }
    let path = path.canonicalize()?;
    if !path.starts_with(&self.root) {
      control.write_all(b"550 Permission denied.\r\n").await?;
      return Ok(());
    }

    let list =
      get_list_lines(&path, name_only).unwrap_or_else(|_| "Something wrong.\r\n".to_string());

    let session = user.get_session()?;
    let mut session = session.lock().await;
    let data_stream = session.get_stream();
    let mut data_stream = data_stream.lock().await;

    control
      .write_all(b"150 Opening ASCII mode data connection for file list\r\n")
      .await?;
    data_stream.write_all(list.as_bytes()).await?;
    data_stream.shutdown().await?;
    session.set_finished(true);
    control.write_all(b"226 Transfer complete.\r\n").await?;
    Ok(())
  }

  async fn store_file(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>> {
    let (target_path, mut offset) = {
      let user = user.lock().await;
      let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
      let session = user.get_session()?;
      let mut session = session.lock().await;
      session.file_name = file_name.clone();

      (path, session.offset)
    };

    if !target_path.starts_with(&self.root) {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await?;
      return Ok(());
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
        .await?;
    }

    let mut file = if target_path.exists() {
      let meta = target_path.metadata()?;
      if meta.is_dir() {
        control
          .lock()
          .await
          .write_all(b"550 Permission denied, the path is a directory.\r\n")
          .await?;
        return Ok(());
      }
      if offset == 0 {
        control
          .lock()
          .await
          .write_all(b"550 Permission denied, the file exists.\r\n")
          .await?;
      }
      if offset > meta.len() {
        offset = meta.len();
      }
      let mut file = fs::File::open(target_path)?;
      file.seek(std::io::SeekFrom::Start(offset))?;
      file
    } else {
      fs::File::create(target_path)?
    };

    loop {
      let user = user.lock().await;
      let session = user.get_session()?;
      let mut session = session.lock().await;

      if session.aborted {
        break;
      }

      let data_stream = session.get_stream();
      let mut data_stream = data_stream.lock().await;

      let mut buf = vec![0; 1024];
      let n = data_stream.read(&mut buf).await?;

      if n == 0 {
        break;
      }
      file.write_all(&buf[..n])?;
      session.finished_size += n as u64;
    }

    let user = user.lock().await;
    let session = user.get_session()?;
    let mut session = session.lock().await;

    let data_stream = session.get_stream();
    let mut data_stream = data_stream.lock().await;
    data_stream.shutdown().await?;
    if session.aborted {
      control
        .lock()
        .await
        .write_all(b"226 Connection closed; transfer aborted.\r\n")
        .await?;
    } else {
      session.finished = true;
      control
        .lock()
        .await
        .write_all(b"226 Transfer complete.\r\n")
        .await?;
    }
    Ok(())
  }
}

fn file_path_to_list_item(path: &PathBuf, name_only: bool) -> Result<String, Box<dyn Error>> {
  // https://files.stairways.com/other/ftp-list-specs-info.txt
  // http://cr.yp.to/ftp/list/binls.html
  let metadata = fs::metadata(&path)?;
  let file_name = match path.file_name() {
    Some(name) => match name.to_str() {
      Some(name) => name,
      None => {
        return Err("Error: file name is not valid UTF-8.".into());
      }
    },
    None => {
      return Err("Error: file name is None.".into());
    }
  };
  if name_only {
    return Ok(format!("{}\r\n", file_name).to_string());
  }
  let file_size = format!("{:>13}", metadata.len());
  let file_type = if metadata.is_dir() { "d" } else { "-" };
  let file_time = metadata
    .modified()?
    .duration_since(std::time::SystemTime::UNIX_EPOCH)?;
  let file_time = DateTime::from_timestamp(file_time.as_secs() as i64, 0)
    .unwrap_or_default()
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
  ) -> Result<(), Box<dyn Error>> {
    self.list_files(control, user, optional_dir, false).await
  }

  async fn name_list(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_dir: Option<String>,
  ) -> Result<(), Box<dyn Error>> {
    self.list_files(control, user, optional_dir, true).await
  }

  async fn retrieve(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>> {
    let (path, offset) = {
      let user = user.lock().await;

      let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
      let session = user.get_session()?;
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
        .await?;
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
        .await?;
    }

    let mut file = fs::File::open(path)?;
    if offset > 0 {
      let meta = file.metadata()?;
      let file_size = meta.len();
      if offset >= file_size {
        control
          .lock()
          .await
          .write_all(b"550 Offset out of range.\r\n")
          .await?;
        return Ok(());
      }
      file.seek(std::io::SeekFrom::Start(offset))?;
    }
    loop {
      let user = user.lock().await;
      let session = user.get_session()?;
      let mut session = session.lock().await;
      if session.aborted {
        break;
      }
      let data_stream = session.get_stream();
      let mut data_stream = data_stream.lock().await;
      let mut buf = vec![0u8; 1024];
      let n = file.read(&mut buf)?;
      if n == 0 {
        break;
      }
      data_stream.write_all(&buf[..n]).await?;
      session.finished_size += n as u64;
    }

    let user = user.lock().await;
    let session = user.get_session()?;
    let mut session = session.lock().await;

    let data_stream = session.get_stream();
    let mut data_stream = data_stream.lock().await;
    data_stream.shutdown().await?;

    if session.aborted {
      control
        .lock()
        .await
        .write_all(b"226 Connection closed; transfer aborted.\r\n")
        .await?;
    } else {
      session.finished = true;
      control
        .lock()
        .await
        .write_all(b"226 Transfer complete.\r\n")
        .await?;
    }
    Ok(())
  }

  async fn store(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>> {
    self.store_file(control, user, file_name).await
  }

  async fn make_dir(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) -> Result<(), Box<dyn Error>> {
    let user = user.lock().await;
    // let parts = current_user.pwd.split("/").collect();
    match fs::create_dir(Path::new(&self.root).join(&user.pwd).join(&dir_name)) {
      Ok(_) => {
        control
          .lock()
          .await
          .write_all(b"257 Directory created.\r\n")
          .await?;
      }
      Err(e) => {
        println!("Error: {:?}", e);
        control
          .lock()
          .await
          .write_all(b"550 Permission denied.\r\n")
          .await?;
      }
    }
    Ok(())
  }
  
  async fn remove_dir(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) -> Result<(), Box<dyn Error>> {
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
            .await?;
        }
        if !new_path.exists() {
          control
            .lock()
            .await
            .write_all(b"553 Not found.\r\n")
            .await?;
        }
        if let Ok(_) = fs::remove_dir(new_path) {
          control
            .lock()
            .await
            .write_all(b"200 Remove completed.\r\n")
            .await?;
        } else {
          control
            .lock()
            .await
            .write_all(b"550 Failed to remove directory.\r\n")
            .await?;
        }
      }
      Err(e) => {
        println!("Error: {:?}", e);
        control
          .lock()
          .await
          .write_all(b"550 Path error.\r\n")
          .await?;
      }
    }
    Ok(())
  }

  async fn delete(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>> {
    let user = user.lock().await;
    let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
    if !path.exists() {
      control
        .lock()
        .await
        .write_all(b"553 Not found.\r\n")
        .await?;
      return Ok(());
    }
    if !path.starts_with(&self.root) {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await?;
      return Ok(());
    }
    match fs::remove_file(path) {
      Ok(_) => {
        control
          .lock()
          .await
          .write_all(b"250 Requested file action okay, completed.\r\n")
          .await?;
      }
      Err(e) => {
        println!("Error: {:?}", e);
        control
          .lock()
          .await
          .write_all(b"550 Failed to remove file.\r\n")
          .await?;
      }
    };
    Ok(())
  }

  async fn cwd(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    dir_name: String,
  ) -> Result<(), Box<dyn Error>> {
    let mut user = user.lock().await;
    let dir_name = dir_name.trim_start_matches("/");
    if dir_name.is_empty() {
      user.pwd = ".".to_string();
      control
        .lock()
        .await
        .write_all(b"250 Requested file action okay, completed.\r\n")
        .await?;
      return Ok(());
    } else if dir_name == "." {
      control
        .lock()
        .await
        .write_all(b"250 PWD not changed.\r\n")
        .await?;
      return Ok(());
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
          .await?;
      }
      if !new_path.starts_with(&self.root) {
        control
          .lock()
          .await
          .write_all(b"550 Permission denied.\r\n")
          .await?;
      }
      user.pwd = match new_path.to_str() {
        Some(p) => p.to_string(),
        None => {
          return Err("Error: path to string failed.".into());
        }
      };
      user.pwd = user.pwd.to_string().replace(&self.root, ".");
      control
        .lock()
        .await
        .write_all(b"250 Requested file action okay, completed.\r\n")
        .await?;
    } else {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await?;
    }
    Ok(())
  }

  async fn pwd(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    let user = user.lock().await;
    control
      .lock()
      .await
      .write_all(format!("257 \"{}\" is the current directory.\r\n", &user.pwd).as_bytes())
      .await?;
    Ok(())
  }

  async fn set_type(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    type_: String,
  ) -> Result<(), Box<dyn Error>> {
    match type_.to_uppercase().as_str() {
      "A" => {
        user.lock().await.trans_type = TransferType::ASCII;
        control
          .lock()
          .await
          .write_all(b"200 Type set to ASCII.\r\n")
          .await?;
      }
      "I" => {
        user.lock().await.trans_type = TransferType::Binary;
        control
          .lock()
          .await
          .write_all(b"200 Type set to Binary.\r\n")
          .await?;
      }
      _ => {
        control
          .lock()
          .await
          .write_all(b"504 Command not implemented for that parameter.\r\n")
          .await?;
      }
    }
    Ok(())
  }

  async fn passive_mode(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    let cloned = user.clone();
    let listener = self.generate_pasv_addr().await?;
    let listen_addr = listener.local_addr().unwrap_or(SocketAddr::from_str(
      format!("{}:{}", self.host, self.port).as_str(),
    )?);
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
      .await?;
    // let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
      let (stream, _) = match listener.accept().await {
        Ok((s, addr)) => (s, addr),
        Err(e) => {
          println!("Listen pasv error: {}", e);
          return;
        }
      };
      cloned
        .lock()
        .await
        .set_new_session(TransferSession::new(TransferMode::Passive(Arc::new(
          Mutex::new(stream),
        ))));
    });
    Ok(())
  }

  async fn port_mode(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    port_addr: SocketAddr,
  ) -> Result<(), Box<dyn Error>> {
    let mut user = user.lock().await;
    let stream = TcpStream::connect(port_addr).await?;

    user.set_new_session(TransferSession::new(TransferMode::Port(Arc::new(
      Mutex::new(stream),
    ))));

    control
      .lock()
      .await
      .write_all(b"200 PORT command successful.\r\n")
      .await?;
    Ok(())
  }

  async fn quit(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    let mut user = user.lock().await;
    user.session = None;
    let mut locking = control.lock().await;
    locking.write_all(b"221 Goodbye.\r\n").await?;
    locking.shutdown().await?;
    Ok(())
  }

  async fn noop(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    _user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    control.lock().await.write_all(b"200 NOOP ok.\r\n").await?;
    Ok(())
  }

  async fn user(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    username: String,
  ) -> Result<(), Box<dyn Error>> {
    let mut user = user.lock().await;
    user.username = username;
    user.status = UserStatus::Logging;
    control
      .lock()
      .await
      .write_all(b"331 User name okay, need password.\r\n")
      .await?;
    Ok(())
  }

  async fn pass(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    _: String,
  ) -> Result<(), Box<dyn Error>> {
    {
      user.lock().await.status = UserStatus::Active;
    }
    control
      .lock()
      .await
      .write_all(b"230 User logged in, proceed.\r\n")
      .await?;
    Ok(())
  }

  async fn abort(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    let mut locking = user.lock().await;
    locking.status = UserStatus::Active;
    let session = locking.get_session()?;
    let mut session = session.lock().await;
    session.aborted = true;
    control
      .lock()
      .await
      .write_all(b"226 ABOR command processed.\r\n") // '426', '225', '226'
      .await?;
    Ok(())
  }

  async fn system_info(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    _user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    control
      .lock()
      .await
      .write_all(b"215 UNIX Type: L8\r\n")
      .await?;
    Ok(())
  }

  async fn rename_from(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>> {
    let user = user.lock().await;
    let session = user.get_session()?;
    let mut session = session.lock().await;
    session.file_name = file_name;
    control
      .lock()
      .await
      .write_all(b"350 Requested file action pending further information.\r\n")
      .await?;
    Ok(())
  }

  async fn rename_to(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>> {
    let user = user.lock().await;
    let pwd = user.pwd.clone();
    let session = user.get_session()?;
    let mut session = session.lock().await;
    let old_path = Path::new(&self.root).join(&pwd).join(&session.file_name);
    let new_path = Path::new(&self.root).join(&pwd).join(&file_name);
    fs::rename(old_path, new_path)?;
    session.file_name = file_name;
    {
      control
        .lock()
        .await
        .write_all(b"250 Requested file action okay, completed.\r\n")
        .await?;
    }
    Ok(())
  }

  async fn restart(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    offset: u64,
  ) -> Result<(), Box<dyn Error>> {
    let user = user.lock().await;
    let session = user.get_session()?;
    let mut session = session.lock().await;
    session.offset = offset;
    control
      .lock()
      .await
      .write_all(b"350 Requested file action pending further information.\r\n")
      .await?;
    Ok(())
  }

  async fn status(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    optional_path: Option<String>,
  ) -> Result<(), Box<dyn Error>> {
    let user = user.lock().await;
    let mut control = control.lock().await;
    match optional_path {
      Some(path_str) => {
        let path = Path::new(&self.root).join(&user.pwd).join(&path_str);
        if !path.exists() {
          control.write_all(b"553 Not found.\r\n").await?;
        } else {
          let path = path.canonicalize()?;
          if !path.starts_with(&self.root) {
            control.write_all(b"550 Permission denied.\r\n").await?;
          }
          let list =
            get_list_lines(&path, false).unwrap_or_else(|_| "Something wrong.\r\n".to_string());
          control
            .write_all(format!("213-Status of {}:\r\n", path_str).as_bytes())
            .await?;
          control.write_all(list.as_bytes()).await?;
          control.write_all(b"213 End of status.\r\n").await?;
        }
      }
      None => {
        control.write_all(b"211-Status of the server:\r\n").await?;
        let mut content = String::new();
        // content.push_str(format!("Server root: {}\r\n", self.root).as_str());
        content.push_str(format!("User: {}\r\n", user.username).as_str());
        content.push_str(format!("Current directory: {}\r\n", user.pwd).as_str());
        content.push_str(format!("TYPE: {:?}\r\n", user.trans_type).as_str());
        control.write_all(b"211 End of status.\r\n").await?;
      }
    }
    Ok(())
  }

  async fn store_unique(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    let file_name = Uuid::new_v4().to_string();
    self.store_file(control, user, file_name).await
  }

  async fn append(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>> {
    {
      let user = user.lock().await;
      let session = user.get_session()?;
      let mut session = session.lock().await;
      session.file_name = file_name.clone();
      session.offset = u64::MAX;
    }
    self.store_file(control, user, file_name).await
  }

  async fn allocate(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    _user: Arc<Mutex<User>>,
    _size: u64,
  ) -> Result<(), Box<dyn Error>> {
    control
      .lock()
      .await
      .write_all(b"200 ALLO command okay.\r\n")
      .await?;
    Ok(())
  }

  async fn feat(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    _user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    let mut locking = control.lock().await;
    locking.write_all(b"211-Features:\r\n").await?;
    locking.write_all(b" REST STREAM\r\n").await?;
    locking.write_all(b" MDTM\r\n").await?;
    locking.write_all(b"211 End.\r\n").await?;
    Ok(())
  }

  async fn cd_up(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
  ) -> Result<(), Box<dyn Error>> {
    let mut user = user.lock().await;
    let mut new_path = Path::new(&self.root).join(&user.pwd);
    if new_path.pop() {
      let new_path = new_path.canonicalize()?;
      if new_path.starts_with(&self.root) {
        user.pwd = new_path
          .to_str()
          .unwrap_or(".")
          .to_string()
          .replace(&self.root, ".");
      } else {
        control
          .lock()
          .await
          .write_all(b"550 Permission denied.\r\n")
          .await?;
      }
    } else {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await?;
    }
    control
      .lock()
      .await
      .write_all(b"250 Directory successfully changed.\r\n")
      .await?;
    Ok(())
  }

  async fn get_modify_timestamp(
    &self,
    control: Arc<Mutex<OwnedWriteHalf>>,
    user: Arc<Mutex<User>>,
    file_name: String,
  ) -> Result<(), Box<dyn Error>> {
    let user = user.lock().await;
    let path = Path::new(&self.root).join(&user.pwd).join(&file_name);
    if !path.exists() {
      control
        .lock()
        .await
        .write_all(b"553 Not found.\r\n")
        .await?;
      return Ok(());
    }
    if !path.starts_with(&self.root) {
      control
        .lock()
        .await
        .write_all(b"550 Permission denied.\r\n")
        .await?;
      return Ok(());
    }
    let metadata = fs::metadata(&path)?;
    let file_time = metadata
      .modified()?
      .duration_since(std::time::SystemTime::UNIX_EPOCH)?;
    let file_time = DateTime::from_timestamp(file_time.as_secs() as i64, 0)
      .unwrap_or_default()
      .with_timezone(&Local)
      .format("%Y%m%d%H:%M%S")
      .to_string();
    control
      .lock()
      .await
      .write_all(format!("213 {}\r\n", file_time).as_bytes())
      .await?;
    Ok(())
  }
}
