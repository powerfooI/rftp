mod arg_parser;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn handle_client(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    let mut content: Vec<String> = Vec::new();
    let (rd, mut wr) = io::split(stream);
    let buf_reader = BufReader::new(rd);
    let mut lines = buf_reader.lines();

    while let Some(line) = lines.next_line().await? {
        if !line.is_empty() {
            content.push(line);
        }
    }

    println!("{:#?}", content.clone());

    wr.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;

    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = arg_parser::Args::parse_args();
    println!("{args:?}");

    let bind_loc = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(bind_loc).await?;
    
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut stream).await {
                println!("{e}");
            };
        });
    }
}
