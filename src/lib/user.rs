use std::{collections::HashMap, net::SocketAddr};
use crate::lib::session::TransferSession;
use tokio::sync::{Mutex, oneshot};

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
  pub pwd: String,
  pub sessions: HashMap<SocketAddr, TransferSession>,
  pub trans_type: TransferType,
  // pub cancel_tx: Option<oneshot::Sender<()>>,
}

impl User {
  pub fn new(username: String, addr: SocketAddr) -> Self {
    Self {
      addr,
      username,
      sessions: HashMap::new(),
      pwd: String::from("."),
      status: UserStatus::Logging,
      trans_type: TransferType::ASCII,
    }
  }
  pub fn new_anonymous(addr: SocketAddr) -> Self {
    Self {
      addr,
      status: UserStatus::Active,
      username: String::from("anonymous"),
      sessions: HashMap::new(),
      pwd: String::from("."),
      trans_type: TransferType::ASCII,
    }
  }
}
