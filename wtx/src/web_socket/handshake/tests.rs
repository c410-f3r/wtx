macro_rules! call_tests {
  (($method:ident, $ws:expr), $($struct:ident),+ $(,)?) => {
    $(
      $struct::$method($ws).await;
      Sleep::new(Duration::from_millis(200)).unwrap().await.unwrap();
    )+
  };
}

use crate::{
  collections::Vector,
  executor::StdRuntime,
  futures::Sleep,
  rng::{ChaCha20, CryptoSeedableRng},
  sync::{Arc, AtomicBool},
  tests::_uri,
  tls::{TlsAcceptor, TlsConfig, TlsConnector, TlsModePlainText},
  web_socket::{
    Frame, OpCode, WebSocket, WebSocketAcceptor, WebSocketConnector, WebSocketPayloadOrigin,
    WsCompression, web_socket_compression::NegotiatedWsCompression,
  },
};
use core::{sync::atomic::Ordering, time::Duration};
use std::net::{TcpListener, TcpStream};

static HAS_SERVER_FINISHED: AtomicBool = AtomicBool::new(false);

type LocalWebSocket<NC, const IS_CLIENT: bool> =
  WebSocket<NC, TcpStream, TlsModePlainText, IS_CLIENT>;

#[cfg_attr(miri, ignore)]
#[cfg(feature = "zlib-rs")]
#[test]
fn compressed() {
  let runtime = Arc::new(StdRuntime::new());
  let runtime_fut0 = runtime.clone();
  let runtime_fut1 = runtime.clone();
  let runtime_fut2 = runtime.clone();
  runtime.block_on(async {
    use crate::web_socket::web_socket_compression::ZlibRs;
    do_test_client_and_server_frames(((), false), (ZlibRs::default(), false), runtime_fut0).await;
    Sleep::new(Duration::from_millis(200)).unwrap().await.unwrap();
    do_test_client_and_server_frames((ZlibRs::default(), false), ((), false), runtime_fut1).await;
    Sleep::new(Duration::from_millis(200)).unwrap().await.unwrap();
    do_test_client_and_server_frames(
      (ZlibRs::default(), false),
      (ZlibRs::default(), false),
      runtime_fut2,
    )
    .await;
  });
}

#[cfg_attr(miri, ignore)]
#[test]
fn uncompressed() {
  let runtime = Arc::new(StdRuntime::new());
  let runtime_fut = runtime.clone();
  runtime.block_on(do_test_client_and_server_frames(((), false), ((), false), runtime_fut));
}

#[cfg_attr(miri, ignore)]
#[test]
fn uncompressed_no_masking() {
  let runtime = Arc::new(StdRuntime::new());
  let runtime_fut = runtime.clone();
  runtime.block_on(do_test_client_and_server_frames(((), true), ((), true), runtime_fut));
}

async fn do_test_client_and_server_frames<CC, SC>(
  (client_compression, client_no_masking): (CC, bool),
  (server_compression, server_no_masking): (SC, bool),
  runtime: Arc<StdRuntime>,
) where
  CC: WsCompression<true> + Send,
  CC::NegotiatedCompression: Send,
  SC: WsCompression<false> + Send + 'static,
  SC::NegotiatedCompression: Send,
  for<'nc> &'nc SC::NegotiatedCompression: Send,
{
  let uri = _uri();

  let listener = TcpListener::bind(uri.hostname_with_implied_port()).unwrap();
  let _fut = runtime
    .spawn(async move {
      let (stream, _) = listener.accept().unwrap();
      let mut ws = WebSocketAcceptor::default()
        .set_compression(server_compression)
        .set_no_masking(server_no_masking)
        .accept(TlsAcceptor::new(
          &TlsConfig::plaintext(),
          &mut ChaCha20::from_std_random().unwrap(),
          stream,
        ))
        .await
        .unwrap();
      call_tests!(
        (server, &mut ws),
        FragmentedText,
        LargeFragmentedText,
        PingAndText,
        PingBetweenFragmentedText,
        TwoPings,
        // Last,
        HelloAndGoodbye,
      );
      HAS_SERVER_FINISHED.store(true, Ordering::Relaxed);
    })
    .unwrap();

  let stream = TcpStream::connect(uri.hostname_with_implied_port()).unwrap();
  let mut ws = WebSocketConnector::default()
    .set_compression(client_compression)
    .set_no_masking(client_no_masking)
    .connect(
      TlsConnector::new(&TlsConfig::plaintext(), &mut ChaCha20::from_std_random().unwrap(), stream),
      &uri.to_ref(),
    )
    .await
    .unwrap();
  call_tests!(
    (client, &mut ws),
    FragmentedText,
    LargeFragmentedText,
    PingAndText,
    PingBetweenFragmentedText,
    TwoPings,
    // Last,
    HelloAndGoodbye,
  );

  let mut has_server_finished = false;
  for _ in 0..15 {
    let local_has_server_finished = HAS_SERVER_FINISHED.load(Ordering::Relaxed);
    if local_has_server_finished {
      has_server_finished = local_has_server_finished;
      break;
    }
    Sleep::new(Duration::from_millis(200)).unwrap().await.unwrap();
  }
  if !has_server_finished {
    panic!("Server didn't finish");
  }
}

trait Test<NC> {
  async fn client(ws: &mut LocalWebSocket<NC, true>);

  async fn server(ws: &mut LocalWebSocket<NC, false>);
}

struct FragmentedText;
impl<NC> Test<NC> for FragmentedText
where
  NC: NegotiatedWsCompression,
{
  async fn client(ws: &mut LocalWebSocket<NC, true>) {
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut [b'1']).unwrap()).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut [b'2', b'3']).unwrap())
      .await
      .unwrap();
  }

  async fn server(ws: &mut LocalWebSocket<NC, false>) {
    let mut buffer = Vector::new();
    let text = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "123".as_bytes()), (text.op_code(), &**text.payload()));
  }
}

struct HelloAndGoodbye;
impl<NC> Test<NC> for HelloAndGoodbye
where
  NC: NegotiatedWsCompression,
{
  async fn client(ws: &mut LocalWebSocket<NC, true>) {
    let mut buffer = Vector::new();
    let hello = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "Hello!".as_bytes()), (hello.op_code(), &**hello.payload()));
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"Goodbye!").unwrap()).await.unwrap();
    assert_eq!(
      ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive)
        .await
        .unwrap()
        .payload()
        .get(2..),
      Some(&b"PS: s2"[..])
    );
  }

  async fn server(ws: &mut LocalWebSocket<NC, false>) {
    let mut buffer = Vector::new();
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"Hello!").unwrap()).await.unwrap();
    assert_eq!(
      ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap().payload(),
      b"Goodbye!"
    );
    let ps = [3, 232, 80, 83, 58, 32, 115, 50];
    ws.write_frame(&mut Frame::new_fin(OpCode::Close, ps).unwrap()).await.unwrap();
  }
}

struct LargeFragmentedText;
impl<NC> Test<NC> for LargeFragmentedText
where
  NC: NegotiatedWsCompression,
{
  async fn client(ws: &mut LocalWebSocket<NC, true>) {
    let bytes = || Vector::from_vec(alloc::vec![b'1'; 256 * 1024]);
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut bytes()).unwrap()).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes()).unwrap())
      .await
      .unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes()).unwrap())
      .await
      .unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes()).unwrap())
      .await
      .unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes()).unwrap())
      .await
      .unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes()).unwrap())
      .await
      .unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes()).unwrap())
      .await
      .unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes()).unwrap())
      .await
      .unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes()).unwrap())
      .await
      .unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut bytes()).unwrap()).await.unwrap();
  }

  async fn server(ws: &mut LocalWebSocket<NC, false>) {
    let mut buffer = Vector::new();
    let text = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!(
      (OpCode::Text, Vector::from_vec(alloc::vec![b'1'; 10 * 256 * 1024]).as_slice().len()),
      (text.op_code(), text.payload().len())
    );
  }
}

struct PingAndText;
impl<NC> Test<NC> for PingAndText
where
  NC: NegotiatedWsCompression,
{
  async fn client(ws: &mut LocalWebSocket<NC, true>) {
    let mut buffer = Vector::new();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, *b"123").unwrap()).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"ipat").unwrap()).await.unwrap();
    assert_eq!(
      OpCode::Pong,
      ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap().op_code()
    );
  }

  async fn server(ws: &mut LocalWebSocket<NC, false>) {
    let mut buffer = Vector::new();
    let frame = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Ping, "123".as_bytes()), (frame.op_code(), &**frame.payload()));
    let frame = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "ipat".as_bytes()), (frame.op_code(), &**frame.payload()));
  }
}

struct PingBetweenFragmentedText;
impl<NC> Test<NC> for PingBetweenFragmentedText
where
  NC: NegotiatedWsCompression,
{
  async fn client(ws: &mut LocalWebSocket<NC, true>) {
    let mut buffer = Vector::new();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut [b'1']).unwrap()).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [b'9']).unwrap()).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut [b'2', b'3']).unwrap())
      .await
      .unwrap();
    assert_eq!(
      OpCode::Pong,
      ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap().op_code()
    );
  }

  async fn server(ws: &mut LocalWebSocket<NC, false>) {
    let mut buffer = Vector::new();
    let frame = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Ping, "9".as_bytes()), (frame.op_code(), &**frame.payload()));
    let frame = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "123".as_bytes()), (frame.op_code(), &**frame.payload()));
  }
}

struct TwoPings;
impl<NC> Test<NC> for TwoPings
where
  NC: NegotiatedWsCompression,
{
  async fn client(ws: &mut LocalWebSocket<NC, true>) {
    let mut buffer = Vector::new();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, *b"0").unwrap()).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, *b"1").unwrap()).await.unwrap();
    let zero = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Pong, "0".as_bytes()), (zero.op_code(), &**zero.payload()));
    let one = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Pong, "1".as_bytes()), (one.op_code(), &**one.payload()));
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"2").unwrap()).await.unwrap();
  }

  async fn server(ws: &mut LocalWebSocket<NC, false>) {
    let mut buffer = Vector::new();
    let zero = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Ping, "0".as_bytes()), (zero.op_code(), &**zero.payload()));
    let one = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Ping, "1".as_bytes()), (one.op_code(), &**one.payload()));
    let two = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "2".as_bytes()), (two.op_code(), &**two.payload()));
  }
}
