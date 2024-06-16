use std::sync::Arc;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, oneshot};

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
}

impl TransferSession {
  pub fn new(mode: TransferMode) -> Self {
    Self {
      mode,
      total_size: 0,
      finished_size: 0,
      file_name: String::new(),
      finished: false,
    }
  }
  pub fn set_file_name(&mut self, name: String) {
    self.file_name = name;
  }
  pub fn set_total_size(&mut self, size: u64) {
    self.total_size = size;
  }
  pub fn set_finished_size(&mut self, size: u64) {
    self.finished_size = size;
  }
  pub fn set_finished(&mut self, finished: bool) {
    self.finished = finished;
  }
  pub fn get_stream(&self) -> Arc<Mutex<TcpStream>> {
    match &self.mode {
      TransferMode::Port(stream) => stream.clone(),
      TransferMode::Passive(stream) => stream.clone(),
    }
  }
}

