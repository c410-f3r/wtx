macro_rules! call_tests {
  (($method:ident, $ws:expr), $($struct:ident),+ $(,)?) => {
    $(
      $struct::$method($ws).await;
      sleep(Duration::from_millis(200)).await.unwrap();
    )+
  };
}

use crate::{
  collection::Vector,
  executor::Runtime,
  misc::sleep,
  rng::Xorshift64,
  sync::{Arc, AtomicBool},
  tests::_uri,
  web_socket::{
    CloseCode, Compression, Frame, OpCode, WebSocketAcceptor, WebSocketConnector, WebSocketOwned,
    WebSocketPayloadOrigin, compression::NegotiatedCompression, fill_with_close_code,
  },
};
use core::{sync::atomic::Ordering, time::Duration};
use std::net::{TcpListener, TcpStream};

static HAS_SERVER_FINISHED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "flate2")]
#[test]
fn compressed() {
  let runtime = Arc::new(Runtime::new());
  let runtime_fut0 = runtime.clone();
  let runtime_fut1 = runtime.clone();
  let runtime_fut2 = runtime.clone();
  runtime
    .block_on(async {
      use crate::web_socket::compression::Flate2;
      do_test_client_and_server_frames(((), false), (Flate2::default(), false), runtime_fut0).await;
      sleep(Duration::from_millis(200)).await.unwrap();
      do_test_client_and_server_frames((Flate2::default(), false), ((), false), runtime_fut1).await;
      sleep(Duration::from_millis(200)).await.unwrap();
      do_test_client_and_server_frames(
        (Flate2::default(), false),
        (Flate2::default(), false),
        runtime_fut2,
      )
      .await;
    })
    .unwrap();
}

#[test]
fn uncompressed() {
  let runtime = Arc::new(Runtime::new());
  let runtime_fut = runtime.clone();
  runtime
    .block_on(do_test_client_and_server_frames(((), false), ((), false), runtime_fut))
    .unwrap();
}

#[test]
fn uncompressed_no_masking() {
  let runtime = Arc::new(Runtime::new());
  let runtime_fut = runtime.clone();
  runtime.block_on(do_test_client_and_server_frames(((), true), ((), true), runtime_fut)).unwrap();
}

async fn do_test_client_and_server_frames<CC, SC>(
  (client_compression, client_no_masking): (CC, bool),
  (server_compression, server_no_masking): (SC, bool),
  runtime: Arc<Runtime>,
) where
  CC: Compression<true> + Send,
  CC::NegotiatedCompression: Send,
  SC: Compression<false> + Send + 'static,
  SC::NegotiatedCompression: Send,
  for<'nc> &'nc SC::NegotiatedCompression: Send,
{
  let uri = _uri();

  let listener = TcpListener::bind(uri.hostname_with_implied_port()).unwrap();
  let _fut = runtime
    .spawn_threaded(async move {
      let (stream, _) = listener.accept().unwrap();
      let mut ws = WebSocketAcceptor::default()
        .compression(server_compression)
        .no_masking(server_no_masking)
        .accept(stream)
        .await
        .unwrap();
      call_tests!(
        (server, &mut ws),
        FragmentedText,
        LargeFragmentedText,
        PingAndText,
        PingBetweenFragmentedText,
        SeveralBytes,
        TwoPings,
        // Last,
        HelloAndGoodbye,
      );
      HAS_SERVER_FINISHED.store(true, Ordering::Relaxed);
    })
    .unwrap();

  let mut ws = WebSocketConnector::default()
    .compression(client_compression)
    .no_masking(client_no_masking)
    .connect(TcpStream::connect(uri.hostname_with_implied_port()).unwrap(), &uri.to_ref())
    .await
    .unwrap();
  call_tests!(
    (client, &mut ws),
    FragmentedText,
    LargeFragmentedText,
    PingAndText,
    PingBetweenFragmentedText,
    SeveralBytes,
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
    sleep(Duration::from_millis(200)).await.unwrap();
  }
  if !has_server_finished {
    panic!("Server didn't finish");
  }
}

trait Test<NC> {
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>);

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>);
}

struct FragmentedText;
impl<NC> Test<NC> for FragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut [b'1'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut [b'2', b'3'])).await.unwrap();
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    let mut buffer = Vector::new();
    let text = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "123".as_bytes()), (text.op_code(), &**text.payload()));
  }
}

struct HelloAndGoodbye;
impl<NC> Test<NC> for HelloAndGoodbye
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    let mut buffer = Vector::new();
    let hello = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "Hello!".as_bytes()), (hello.op_code(), &**hello.payload()));
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"Goodbye!")).await.unwrap();
    assert_eq!(
      ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive)
        .await
        .unwrap()
        .payload()
        .get(2..),
      Some(&b"PS: s2"[..])
    );
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    let mut buffer = Vector::new();
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"Hello!")).await.unwrap();
    assert_eq!(
      ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap().payload(),
      b"Goodbye!"
    );
    let mut ps = *b"__PS: s2";
    fill_with_close_code(CloseCode::Normal, &mut ps);
    ws.write_frame(&mut Frame::new_fin(OpCode::Close, ps)).await.unwrap();
  }
}

struct LargeFragmentedText;
impl<NC> Test<NC> for LargeFragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    let bytes = || Vector::from_vec(alloc::vec![b'1'; 256 * 1024]);
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut bytes())).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut bytes())).await.unwrap();
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
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
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    let mut buffer = Vector::new();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, *b"123")).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"ipat")).await.unwrap();
    assert_eq!(
      OpCode::Pong,
      ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap().op_code()
    );
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
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
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    let mut buffer = Vector::new();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut [b'1'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [b'9'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut [b'2', b'3'])).await.unwrap();
    assert_eq!(
      OpCode::Pong,
      ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap().op_code()
    );
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    let mut buffer = Vector::new();
    let frame = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Ping, "9".as_bytes()), (frame.op_code(), &**frame.payload()));
    let frame = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "123".as_bytes()), (frame.op_code(), &**frame.payload()));
  }
}

struct SeveralBytes;
impl<NC> Test<NC> for SeveralBytes
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut [206])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [186])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [225])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [189])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [185])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [207])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [131])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [206])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [188])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [206])).await.unwrap();
    ws.write_frame(&mut Frame::new_unfin(OpCode::Continuation, &mut [181])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut [])).await.unwrap();
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    let mut buffer = Vector::new();
    let text = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "κόσμε".as_bytes()), (text.op_code(), &**text.payload()));
  }
}

struct TwoPings;
impl<NC> Test<NC> for TwoPings
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    let mut buffer = Vector::new();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, *b"0")).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, *b"1")).await.unwrap();
    let zero = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Pong, "0".as_bytes()), (zero.op_code(), &**zero.payload()));
    let one = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Pong, "1".as_bytes()), (one.op_code(), &**one.payload()));
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"2")).await.unwrap();
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    let mut buffer = Vector::new();
    let zero = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Ping, "0".as_bytes()), (zero.op_code(), &**zero.payload()));
    let one = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Ping, "1".as_bytes()), (one.op_code(), &**one.payload()));
    let two = ws.read_frame(&mut buffer, WebSocketPayloadOrigin::Adaptive).await.unwrap();
    assert_eq!((OpCode::Text, "2".as_bytes()), (two.op_code(), &**two.payload()));
  }
}
