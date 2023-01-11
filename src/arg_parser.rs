use clap::Parser;

/// Naive FTP server in Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Folder path to serve
    #[arg(long, default_value_t = String::from("./"))]
    pub folder: String,

    /// Listening host
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    /// Listening port
    #[arg(long, default_value_t = 21)]
    pub port: u8,
}

impl Args {
    pub fn parse_args() -> Args {
        self::Parser::parse()
    }
}
