use std::net::SocketAddr;
use tokio::net::{TcpStream};

/// deprecated
#[derive(Debug)]
pub struct SocketMsg {
  pub socket: Box<TcpStream>,
  pub addr: SocketAddr,
}