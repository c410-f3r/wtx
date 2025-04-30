macro_rules! call_tests {
  (($method:ident, $ws:expr), $($struct:ident),+ $(,)?) => {
    $(
      $struct::$method($ws).await;
      tokio::time::sleep(Duration::from_millis(200)).await;
    )+
  };
}

use crate::{
  rng::Xorshift64,
  sync::{AtomicBool, Ordering},
  tests::_uri,
  web_socket::{
    Compression, Frame, OpCode, WebSocketAcceptor, WebSocketConnector, WebSocketOwned,
    compression::NegotiatedCompression,
  },
};
use core::time::Duration;
use tokio::net::{TcpListener, TcpStream};

static HAS_SERVER_FINISHED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "flate2")]
#[tokio::test]
async fn compressed() {
  use crate::web_socket::compression::Flate2;
  #[cfg(feature = "_tracing-tree")]
  let _rslt = crate::misc::tracing_tree_init(None);
  do_test_client_and_server_frames(((), false), (Flate2::default(), false)).await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  do_test_client_and_server_frames((Flate2::default(), false), ((), false)).await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  do_test_client_and_server_frames((Flate2::default(), false), (Flate2::default(), false)).await;
}

#[tokio::test]
async fn uncompressed() {
  #[cfg(feature = "_tracing-tree")]
  let _rslt = crate::misc::tracing_tree_init(None);
  do_test_client_and_server_frames(((), false), ((), false)).await;
}

#[tokio::test]
async fn uncompressed_no_masking() {
  #[cfg(feature = "_tracing-tree")]
  let _rslt = crate::misc::tracing_tree_init(None);
  do_test_client_and_server_frames(((), true), ((), true)).await;
}

async fn do_test_client_and_server_frames<CC, SC>(
  (client_compression, client_no_masking): (CC, bool),
  (server_compression, server_no_masking): (SC, bool),
) where
  CC: Compression<true> + Send,
  CC::NegotiatedCompression: Send,
  SC: Compression<false> + Send + 'static,
  SC::NegotiatedCompression: Send,
  for<'nc> &'nc SC::NegotiatedCompression: Send,
{
  let uri = _uri();

  let listener = TcpListener::bind(uri.hostname_with_implied_port()).await.unwrap();
  let _server_jh = tokio::spawn(async move {
    let (stream, _) = listener.accept().await.unwrap();
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
  });

  let mut ws = WebSocketConnector::default()
    .compression(client_compression)
    .no_masking(client_no_masking)
    .connect(TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap(), &uri.to_ref())
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
    tokio::time::sleep(Duration::from_millis(200)).await;
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
    let text = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!(b"123", text.payload());
  }
}

struct HelloAndGoodbye;
impl<NC> Test<NC> for HelloAndGoodbye
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    let hello = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, hello.op_code());
    assert_eq!(b"Hello!", hello.payload());
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"Goodbye!")).await.unwrap();
    assert_eq!(OpCode::Close, ws.read_frame().await.unwrap().op_code());
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"Hello!")).await.unwrap();
    assert_eq!(ws.read_frame().await.unwrap().payload(), b"Goodbye!");
    ws.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await.unwrap();
  }
}

struct LargeFragmentedText;
impl<NC> Test<NC> for LargeFragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    let bytes = || vector![b'1'; 256 * 1024];
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
    let text = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!(vector![b'1'; 10 * 256 * 1024].as_slice(), *text.payload());
  }
}

struct PingAndText;
impl<NC> Test<NC> for PingAndText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [1, 2, 3])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, *b"ipat")).await.unwrap();
    assert_eq!(OpCode::Pong, ws.read_frame().await.unwrap().op_code());
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    assert_eq!(b"ipat", ws.read_frame().await.unwrap().payload());
  }
}

struct PingBetweenFragmentedText;
impl<NC> Test<NC> for PingBetweenFragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut [b'1'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [b'9'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut [b'2', b'3'])).await.unwrap();
    assert_eq!(OpCode::Pong, ws.read_frame().await.unwrap().op_code());
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    let text = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!(b"123", text.payload());
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
    let text = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!("κόσμε".as_bytes(), *text.payload());
  }
}

struct TwoPings;
impl<NC> Test<NC> for TwoPings
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, true>) {
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [b'0'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [b'1'])).await.unwrap();
    let zero = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Pong, zero.op_code());
    assert_eq!(b"0", zero.payload());
    let one = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Pong, one.op_code());
    assert_eq!(b"1", one.payload());
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, &mut [])).await.unwrap();
  }

  async fn server(ws: &mut WebSocketOwned<NC, Xorshift64, TcpStream, false>) {
    let zero = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, zero.op_code());
    assert_eq!(b"", zero.payload());
  }
}
