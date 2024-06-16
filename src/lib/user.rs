use crate::lib::session::TransferSession;
use std::sync::Arc;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::{oneshot, Mutex};

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
  pub session: Option<Arc<Mutex<TransferSession>>>,
  pub trans_type: TransferType,
}

impl User {
  pub fn new(username: String, addr: SocketAddr) -> Self {
    Self {
      addr,
      username,
      session: None,
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
      session: None,
      pwd: String::from("."),
      trans_type: TransferType::ASCII,
    }
  }

  pub fn set_new_session(&mut self, session: TransferSession) {
    self.session = Some(Arc::new(Mutex::new(session)));
  }

  pub fn get_session(&self) -> Option<Arc<Mutex<TransferSession>>> {
    self.session.clone()
  }
}
