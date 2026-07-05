//! TLS client that reads and writes records in different tasks.
//!
//! Special records aren't automatically handled by the system in concurrent scenarios because
//! there are multiple ways to synchronize resources. In this example, special records are managed
//! using a channel but you can utilize any other method.
//!
//! `wtx-socket-client-concurrent` is an example that uses a mutex.

extern crate tokio;
extern crate wtx;

use tokio::{net::TcpStream, sync::mpsc::unbounded_channel};
use wtx::{
  collections::ArrayVectorCopy,
  rng::{ChaCha20, CryptoSeedableRng as _},
  stream::{Stream, StreamReader, StreamWriter},
  tls::{TlsConfig, TlsConnector, TlsModeVerified},
};

#[tokio::main]
async fn main() -> wtx::Result<()> {
  let stream = TcpStream::connect("github.com:443").await?;
  let tls_config = TlsConfig::from_ccadb(TlsModeVerified::default())?;
  let tls_connector = TlsConnector::new(tls_config, ChaCha20::from_getrandom()?, stream);
  let tls_stream = tls_connector.connect().await?.tls_stream;
  let (stream_bridge, mut stream_reader, mut stream_writer) = tls_stream.into_split()?;
  let (sender, mut receiver) = unbounded_channel();

  let reader_fut = async {
    loop {
      let mut buffer = ArrayVectorCopy::<u8, 128>::from_array([0; 128]);
      let Some(read) = stream_reader.read(buffer.as_slice_mut().into()).await? else {
        break;
      };
      buffer.truncate(read.get().try_into()?);
      if sender.send(buffer).is_err() {
        break;
      }
    }
    wtx::Result::Ok(())
  };

  let writer_fut = async {
    let request = b"GET /c410-f3r/wtx HTTP/1.1\r\nHost: github.com\r\nConnection: close\r\n\r\n";
    stream_writer.write_all(request).await?;
    loop {
      tokio::select! {
        bridge_opt = stream_bridge.listen() => {
          if let Some(bridge) = bridge_opt {
            stream_writer.manage_bridge_data(bridge).await?;
          } else {
            break;
          }
        }
        receiver_opt = receiver.recv() => {
          if let Some(receiver) = receiver_opt {
            println!("Received data: {receiver:?}");
          } else {
            break;
          }
        }
      }
    }
    wtx::Result::Ok(())
  };

  let (reader_rslt, writer_rslt) = tokio::join!(reader_fut, writer_fut);
  reader_rslt?;
  writer_rslt?;
  Ok(())
}
