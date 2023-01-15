use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{WriteHalf, ReadHalf, BufReader, BufWriter};
use std::sync::{Arc};
use tokio::sync::{Mutex};

#[derive(Debug)]
pub struct SocketMsg {
  pub socket: TcpStream,
  pub addr: SocketAddr,
}