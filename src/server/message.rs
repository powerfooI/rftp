use std::net::SocketAddr;

/**
  USER <SP> <username> <CRLF>
  PASS <SP> <password> <CRLF>
  ACCT <SP> <account-information> <CRLF>
  CWD  <SP> <pathname> <CRLF>
  CDUP <CRLF>
  SMNT <SP> <pathname> <CRLF>
  QUIT <CRLF>
  REIN <CRLF>
  PORT <SP> <host-port> <CRLF>
  PASV <CRLF>
  TYPE <SP> <type-code> <CRLF>
  STRU <SP> <structure-code> <CRLF>
  MODE <SP> <mode-code> <CRLF>
  RETR <SP> <pathname> <CRLF>
  STOR <SP> <pathname> <CRLF>
  STOU <CRLF>
  APPE <SP> <pathname> <CRLF>
  ALLO <SP> <decimal-integer>
      [<SP> R <SP> <decimal-integer>] <CRLF>
  REST <SP> <marker> <CRLF>
  RNFR <SP> <pathname> <CRLF>
  RNTO <SP> <pathname> <CRLF>
  ABOR <CRLF>
  DELE <SP> <pathname> <CRLF>
  RMD  <SP> <pathname> <CRLF>
  MKD  <SP> <pathname> <CRLF>
  PWD  <CRLF>
  LIST [<SP> <pathname>] <CRLF>
  NLST [<SP> <pathname>] <CRLF>
  SITE <SP> <string> <CRLF>
  SYST <CRLF>
  STAT [<SP> <pathname>] <CRLF>
  HELP [<SP> <string>] <CRLF>
  NOOP <CRLF>
 */

#[derive(Debug, Clone)]
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
  TYPE,
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
  STAT,
  STOU,
  APPE(String),
  ALLO(u64),
  NOOP,
  FEAT,
}

pub fn parse_command(req: String) -> FtpCommand {
  let req = req.trim();
  let mut iter = req.split_whitespace();
  let cmd = iter.next().unwrap();
  let arg = iter.collect::<Vec<&str>>().join(" ");
  match cmd {
    "USER" => FtpCommand::USER(arg.to_string()),
    "PASS" => FtpCommand::PASS(arg.to_string()),
    "PORT" => {
      let mut iter = arg.split(',').map(|s| s.parse::<u8>().unwrap());
      let ip = iter.by_ref().take(4).map(|i| i.to_string()).collect::<Vec<String>>().join(".");
      let p1 = iter.next().unwrap() as u16;
      let p2 = iter.next().unwrap() as u16;
      let port = p1 * 256 + p2; // FTP uses two bytes for the port number
      let addr = format!("{}:{}", ip, port);
      FtpCommand::PORT(addr.parse().unwrap())
    }
    "PASV" => {
      FtpCommand::PASV
    }
    "RETR" => FtpCommand::RETR(arg.to_string()),
    "STOR" => FtpCommand::STOR(arg.to_string()),
    "ABOR" => FtpCommand::ABOR,
    "QUIT" => FtpCommand::QUIT,
    "SYST" => FtpCommand::SYST,
    "TYPE" => FtpCommand::TYPE,
    "RNFR" => FtpCommand::RNFR(arg.to_string()),
    "RNTO" => FtpCommand::RNTO(arg.to_string()),
    "PWD" => FtpCommand::PWD,
    "CWD" => FtpCommand::CWD(arg.to_string()),
    "MKD" => FtpCommand::MKD(arg.to_string()),
    "RMD" => FtpCommand::RMD(arg.to_string()),
    "LIST" => {
      if arg.is_empty() {
        FtpCommand::LIST(None)
      } else {
        FtpCommand::LIST(Some(arg.to_string()))
      }
    },
    "REST" => FtpCommand::REST,
    "DELE" => FtpCommand::DELE(arg.to_string()),
    "STAT" => FtpCommand::STAT,
    "STOU" => FtpCommand::STOU,
    "APPE" => FtpCommand::APPE(arg.to_string()),
    "ALLO" => FtpCommand::ALLO(arg.parse().unwrap()),
    "FEAT" => FtpCommand::FEAT,
    _ => {
      println!("Unknown command: {}, Args: {}", cmd, arg);
      FtpCommand::NOOP
    },
  }
}