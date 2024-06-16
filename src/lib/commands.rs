use std::net::SocketAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FtpCommand {
  // Basic commands
  USER(String),
  PASS(String),
  PORT(SocketAddr),
  PASV,
  RETR(String),
  STOR(String),
  ABOR,
  QUIT,
  SYST,
  TYPE(String),
  RNFR(String),
  RNTO(String),
  PWD,
  CWD(String),
  MKD(String),
  RMD(String),
  LIST(Option<String>),

  // Advanced commands
  REST,
  DELE(String),
  STAT(Option<String>),
  STOU,
  APPE(String),
  ALLO(u64),
  NOOP,
  NLST(Option<String>),
  CDUP,

  FEAT,
  MDTM(String),
}

fn empty_to_some(s: String) -> Option<String> {
  if s.is_empty() {
    None
  } else {
    Some(s.to_string())
  }
}

pub fn parse_command(req: String) -> FtpCommand {
  let req = req.trim();
  let mut iter = req.split_whitespace();
  let cmd = iter.next().unwrap();
  let arg = iter.collect::<Vec<&str>>().join(" ");
  match cmd {
    "USER" => FtpCommand::USER(arg),
    "PASS" => FtpCommand::PASS(arg),
    "PORT" => {
      let mut iter = arg.split(',').map(|s| s.parse::<u8>().unwrap());
      let ip = iter
        .by_ref()
        .take(4)
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join(".");
      let p1 = iter.next().unwrap() as u16;
      let p2 = iter.next().unwrap() as u16;
      let port = p1 * 256 + p2; // FTP uses two bytes for the port number
      let addr = format!("{}:{}", ip, port);
      FtpCommand::PORT(addr.parse().unwrap())
    }
    "PASV" => FtpCommand::PASV,
    "RETR" => FtpCommand::RETR(arg),
    "STOR" => FtpCommand::STOR(arg),
    "ABOR" => FtpCommand::ABOR,
    "QUIT" => FtpCommand::QUIT,
    "SYST" => FtpCommand::SYST,
    "TYPE" => FtpCommand::TYPE(arg),
    "RNFR" => FtpCommand::RNFR(arg),
    "RNTO" => FtpCommand::RNTO(arg),
    "PWD" => FtpCommand::PWD,
    "CWD" => FtpCommand::CWD(arg),
    "MKD" => FtpCommand::MKD(arg),
    "RMD" => FtpCommand::RMD(arg),
    "LIST" => FtpCommand::LIST(empty_to_some(arg)),
    "REST" => FtpCommand::REST,
    "DELE" => FtpCommand::DELE(arg),
    "STAT" => FtpCommand::STAT(empty_to_some(arg)),
    "STOU" => FtpCommand::STOU,
    "APPE" => FtpCommand::APPE(arg),
    "ALLO" => FtpCommand::ALLO(arg.parse().unwrap()),
    "FEAT" => FtpCommand::FEAT,
    "CDUP" => FtpCommand::CDUP,
    "MDTM" => FtpCommand::MDTM(arg),
    "NLST" => FtpCommand::NLST(empty_to_some(arg)),
    _ => {
      println!("Unknown command: {}, Args: {}", cmd, arg);
      FtpCommand::NOOP
    }
  }
}
