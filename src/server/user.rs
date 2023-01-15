use std::{net::SocketAddr, collections::HashMap};

#[derive(Debug)]
pub enum UserStatus {
  Inactive,
  Logging,
  Active,
}

#[derive(Debug)]
pub enum TransferMode {
  Port(SocketAddr),
  Passive(SocketAddr),
}

#[derive(Debug)]
pub struct TransferSession {
  pub mode: TransferMode,
  pub total_size: u64,
  pub finished_size: u64,
  pub file_name: String,
}

#[derive(Debug)]
pub struct User {
  pub username: String,
  pub status: UserStatus,
  pub addr: SocketAddr,
  pub sessions: HashMap<SocketAddr, TransferSession>,
}

impl User {
  pub fn new(username: String, addr: SocketAddr) -> Self {
    Self {
      addr,
      username,
      sessions: HashMap::new(),
      status: UserStatus::Logging,
    }
  }
  pub fn new_anonymous(addr: SocketAddr) -> Self {
    Self {
      addr,
      status: UserStatus::Active,
      username: String::from("anonymous"),
      sessions: HashMap::new(),
    }
  }
}
