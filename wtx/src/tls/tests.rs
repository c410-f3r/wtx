use crate::{
  executor::StdRuntime,
  rng::{ChaCha20, CryptoSeedableRng},
  stream::{StreamReader, StreamWriter},
  tests::{_PUBLIC_KEY, _ROOT_CA, _SECRET_KEY, _uri},
  tls::{TlsAcceptor, TlsConfig, TlsConnectorBuilder, TlsModeUnverified},
};
use std::net::TcpListener;

const TM: TlsModeUnverified = TlsModeUnverified::new();

#[cfg_attr(miri, ignore)]
#[wtx::test]
async fn simple_connection(runtime: &StdRuntime) {
  let uri = _uri();
  let mut client_rng = ChaCha20::from_std_random().unwrap();
  let mut server_rng = ChaCha20::from_crypto_rng(&mut client_rng).unwrap();

  let listener = TcpListener::bind(uri.hostname_with_implied_port()).unwrap();

  let _client_jh = runtime
    .spawn(async move {
      let mut tls_stream = TlsConnectorBuilder::std(uri)
        .build(TlsConfig::from_trust_anchors_pem(TM, [_ROOT_CA]).unwrap(), &mut client_rng)
        .await
        .unwrap()
        .connect()
        .await
        .unwrap()
        .tls_stream;
      tls_stream.write_all(b"hello").await.unwrap();
      tls_stream.send_close_notify().await.unwrap();
    })
    .unwrap();

  let stream = listener.accept().unwrap().0;
  let mut tls_stream = TlsAcceptor::new(
    TlsConfig::from_keys_pem(TM, _PUBLIC_KEY, _SECRET_KEY).unwrap(),
    &mut server_rng,
    stream,
  )
  .accept()
  .await
  .unwrap()
  .tls_stream;
  let mut buffer = [0; 128];
  loop {
    let Some(read) = tls_stream.read(buffer.as_mut_slice().into()).await.unwrap() else {
      break;
    };
    let slice = buffer.get(..read.get()).unwrap();
    assert_eq!(slice, b"hello");
  }
}
