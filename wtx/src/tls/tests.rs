use crate::{
  calendar::Instant,
  executor::StdRuntime,
  rng::{ChaCha20, CryptoSeedableRng},
  stream::{StreamReader, StreamWriter},
  tests::_uri,
  tls::{TlsAcceptor, TlsConfig, TlsConnector, TlsModeUnverified},
};
use std::net::{TcpListener, TcpStream};

const TM: TlsModeUnverified = TlsModeUnverified::new();

#[cfg_attr(miri, ignore)]
#[wtx::test]
async fn simple_connection(runtime: &StdRuntime) {
  let uri = _uri();
  let now = Instant::now_date_time(0).unwrap();
  let mut client_rng = ChaCha20::from_std_random().unwrap();
  let mut server_rng = ChaCha20::from_crypto_rng(&mut client_rng).unwrap();

  let listener = TcpListener::bind(uri.hostname_with_implied_port()).unwrap();

  let _client_jh = runtime
    .spawn(async move {
      let stream = TcpStream::connect(uri.hostname_with_implied_port()).unwrap();
      let mut tls_stream = TlsConnector::new(TlsConfig::new(TM, now), &mut client_rng, stream)
        .connect()
        .await
        .unwrap()
        .rslt()
        .unwrap()
        .tls_stream;
      tls_stream.write_all(b"hello").await.unwrap();
      tls_stream.send_close_notify().await.unwrap();
    })
    .unwrap();

  let stream = listener.accept().unwrap().0;
  let mut tls_stream = TlsAcceptor::new(TlsConfig::new(TM, now), &mut server_rng, stream)
    .accept()
    .await
    .unwrap()
    .rslt()
    .unwrap()
    .tls_stream;
  let mut buffer = [0; 128];
  loop {
    let Some(read) = tls_stream.read(buffer.as_mut_slice().into()).await.unwrap().opt() else {
      break;
    };
    let slice = buffer.get(..read.get()).unwrap();
    assert_eq!(slice, b"hello");
  }
}
