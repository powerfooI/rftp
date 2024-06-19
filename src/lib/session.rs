use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum TransferMode {
  Port(Arc<Mutex<TcpStream>>),
  Passive(Arc<Mutex<TcpStream>>),
}

#[derive(Debug)]
pub struct TransferSession {
  pub mode: TransferMode,
  pub total_size: u64,
  pub finished_size: u64,
  pub file_name: String,
  pub finished: bool,
  pub aborted: bool,
  pub offset: u64,
}

impl TransferSession {
  pub fn new(mode: TransferMode) -> Self {
    Self {
      mode,
      total_size: 0,
      finished_size: 0,
      file_name: String::new(),
      finished: false,
      aborted: false,
      offset: 0,
    }
  }
  pub fn get_stream(&self) -> Arc<Mutex<TcpStream>> {
    match &self.mode {
      TransferMode::Port(stream) => stream.clone(),
      TransferMode::Passive(stream) => stream.clone(),
    }
  }
}
