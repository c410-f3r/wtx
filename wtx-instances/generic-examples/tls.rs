use tokio::net::TcpStream;
use wtx::{misc::StreamWriter as _, tls::TlsStream};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let stream = TcpStream::connect("SOME_HTTP1/1_SERVER_ADDR").await?;
  let mut tls_stream = TlsStream::<_, _, true>::new((), stream);
  tls_stream.write_all(b"GET / HTTP/1.1 Host: localhost").await?;
  Ok(())
}
