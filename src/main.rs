mod arg_parser;
mod lib;

use lib::server::Server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  let args = arg_parser::Args::parse_args();
  println!("{args:?}");

  let server = Server::new(args).await?;
  server.listen().await;

  Ok(())
}
