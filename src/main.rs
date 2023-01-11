mod arg_parser;
use std::{
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn handle_client(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let buf_reader = BufReader::new(&mut stream);

    let buf: Vec<_> = buf_reader
        .lines()
        .map(|l| l.unwrap())
        .take_while(|l| !l.is_empty())
        .collect();
    println!("{buf:#?}");

    stream.write_all(b"ok")
}

fn main() -> std::io::Result<()> {
    let args = arg_parser::Args::parse_args();
    println!("{args:?}");

    let listener = TcpListener::bind("127.0.0.1:8180")?;
    // accept connections and process them serially
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        if let Err(e) = handle_client(stream) {
            println!("{e}");
        }
    }
    Ok(())
}
