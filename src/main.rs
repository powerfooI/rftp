mod arg_parser;
mod server;


#[tokio::main]
async fn main() -> std::io::Result<()> {
  let args = arg_parser::Args::parse_args();
  println!("{args:?}");

  let server = server::Server::new(args).await?;
  server.listen().await;

  Ok(())
}
