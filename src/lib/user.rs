use crate::lib::session::TransferSession;
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum UserStatus {
  Inactive,
  Logging,
  Active,
}

#[derive(Debug)]
pub enum TransferType {
  ASCII,
  Binary,
}

#[derive(Debug)]
pub struct User {
  pub username: String,
  pub status: UserStatus,
  pub addr: SocketAddr,
  pub session: Option<Arc<Mutex<TransferSession>>>,
  pub trans_type: TransferType,
  
  path: PathGuard,
}

impl User {
  pub fn cwd(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
    self.path.cwd(path)
  }

  pub fn pwd(&self) -> String {
    self.path.pwd().replace("/", "")
  }
  
  pub fn rendering_pwd(&self) -> String {
    self.path.pwd()
  }

  pub fn new(username: String, addr: SocketAddr, root: &String) -> Result<Self, Box<dyn Error>>{
    Ok(Self {
      addr,
      username,
      session: None,
      path: PathGuard::new(root)?,
      status: UserStatus::Logging,
      trans_type: TransferType::ASCII,
    })
  }

  pub fn new_anonymous(addr: SocketAddr, root: &String) -> Result<Self, Box<dyn Error>> {
    Ok(Self {
      addr,
      username: String::from("anonymous"),
      session: None,
      path: PathGuard::new(root)?,
      status: UserStatus::Active,
      trans_type: TransferType::ASCII,
    })
  }

  pub fn set_new_session(&mut self, session: TransferSession) {
    self.session = Some(Arc::new(Mutex::new(session)));
  }

  pub fn get_session(&self) -> Result<Arc<Mutex<TransferSession>>, Box<dyn Error>> {
    match self.session.clone() {
      Some(s) => Ok(s),
      None => Err("No session found".into()),
    }
  }
}

#[derive(Debug)]
struct PathGuard {
  root: String,
  pub pwd: String,
}

impl PathGuard {
  pub fn new(root: &String) -> Result<Self, Box<dyn Error>> {
    Ok(Self {
      root: match Path::new(root.as_str()).canonicalize()?.to_str() {
        Some(s) => s.to_string(),
        None => return Err("Invalid root path".into()),
      },
      pwd: String::new(),
    })
  }

  pub fn cwd(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
    if path == "." {
      return Ok(());
    }

    let mut path_buf = if path.starts_with("/") {
      Path::new(self.root.as_str()).join(path.trim_start_matches("/"))
    } else {
      Path::new(self.root.as_str())
        .join(self.pwd.as_str())
        .join(path)
    };

    if !path_buf.exists() {
      return Err("Path not found".into());
    }

    path_buf = path_buf.canonicalize()?;

    if !path_buf.starts_with(self.root.as_str()) {
      return Err("Path not allowed".into());
    }
    self.pwd = path_buf
      .to_str()
      .unwrap()
      .to_string()
      .replace(self.root.as_str(), "")
      .trim_start_matches("/")
      .to_string();
    Ok(())
  }

  pub fn pwd(&self) -> String {
    if self.pwd == "" {
      "/".to_string()
    } else {
      self.pwd.clone()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_path_guard() {
    let test_root = String::from("/tmp/test_root");
    let child_folder = String::from("/tmp/test_root/test/test");

    if let Err(e) = std::fs::create_dir_all(&child_folder) {
      panic!("Failed to create test root: {}", e);
    }

    let mut pg = PathGuard::new(&test_root).unwrap();
    assert_eq!(pg.pwd(), "/");

    pg.cwd("test").unwrap();
    assert_eq!(pg.pwd(), "test");
    pg.cwd("test").unwrap();
    assert_eq!(pg.pwd(), "test/test");
    pg.cwd("..").unwrap();
    assert_eq!(pg.pwd(), "test");
    pg.cwd("..").unwrap();
    assert_eq!(pg.pwd(), "/");
    pg.cwd("test").unwrap();
    assert_eq!(pg.pwd(), "test");
    pg.cwd("/").unwrap();
    assert_eq!(pg.pwd(), "/");
    pg.cwd("test").unwrap();
    assert_eq!(pg.pwd(), "test");
    pg.cwd("/test").unwrap();
    assert_eq!(pg.pwd(), "test");
    pg.cwd("/").unwrap();
    assert_eq!(pg.pwd(), "/");
    pg.cwd("test").unwrap();
    assert_eq!(pg.pwd(), "test");
    pg.cwd("/test/test").unwrap();
    assert_eq!(pg.pwd(), "test/test");
    pg.cwd("/test/..").unwrap();
    assert_eq!(pg.pwd(), "/");

    pg.cwd("test3").unwrap_err();
    pg.cwd("/tmp").unwrap_err();

  }
}
