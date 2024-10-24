macro_rules! call_tests {
  (($method:ident, $ws:expr), $($struct:ident),+ $(,)?) => {
    $(
      $struct::$method($ws).await;
      tokio::time::sleep(Duration::from_millis(200)).await;
    )+
  };
}

use crate::{
  misc::{simple_seed, Xorshift64},
  tests::_uri,
  web_socket::{
    compression::NegotiatedCompression, Compression, Frame, OpCode, WebSocketBuffer,
    WebSocketClient, WebSocketClientOwned, WebSocketServer, WebSocketServerOwned,
  },
};
use alloc::vec;
use core::{
  sync::atomic::{AtomicBool, Ordering},
  time::Duration,
};
use tokio::net::{TcpListener, TcpStream};

static HAS_SERVER_FINISHED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "flate2")]
#[tokio::test]
async fn client_and_server_compressed() {
  use crate::web_socket::compression::Flate2;
  #[cfg(feature = "_tracing-tree")]
  let _rslt = crate::misc::tracing_tree_init(None);
  do_test_client_and_server_frames((), Flate2::default()).await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  do_test_client_and_server_frames(Flate2::default(), ()).await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  do_test_client_and_server_frames(Flate2::default(), Flate2::default()).await;
}

#[tokio::test]
async fn client_and_server_uncompressed() {
  #[cfg(feature = "_tracing-tree")]
  let _rslt = crate::misc::tracing_tree_init(None);
  do_test_client_and_server_frames((), ()).await;
}

async fn do_test_client_and_server_frames<CC, SC>(client_compression: CC, server_compression: SC)
where
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
    let mut ws = WebSocketServer::accept(
      server_compression,
      Xorshift64::from(simple_seed()),
      stream,
      WebSocketBuffer::new(),
      |_| crate::Result::Ok(()),
    )
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

  let mut ws = WebSocketClient::connect(
    client_compression,
    [],
    Xorshift64::from(simple_seed()),
    TcpStream::connect(uri.hostname_with_implied_port()).await.unwrap(),
    &uri.to_ref(),
    WebSocketBuffer::new(),
    |_| crate::Result::Ok(()),
  )
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
  async fn client(ws: &mut WebSocketClientOwned<NC, TcpStream>);

  async fn server(ws: &mut WebSocketServerOwned<NC, TcpStream>);
}

struct FragmentedText;
impl<NC> Test<NC> for FragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketClientOwned<NC, TcpStream>) {
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut [b'1'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut [b'2', b'3'])).await.unwrap();
  }

  async fn server(ws: &mut WebSocketServerOwned<NC, TcpStream>) {
    let text = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!(&[b'1', b'2', b'3'], text.payload());
  }
}

struct HelloAndGoodbye;
impl<NC> Test<NC> for HelloAndGoodbye
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketClientOwned<NC, TcpStream>) {
    let hello = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, hello.op_code());
    assert_eq!(b"Hello!", hello.payload());
    ws.write_frame(&mut Frame::new_fin(
      OpCode::Text,
      &mut [b'G', b'o', b'o', b'd', b'b', b'y', b'e', b'!'],
    ))
    .await
    .unwrap();
    assert_eq!(OpCode::Close, ws.read_frame().await.unwrap().op_code());
  }

  async fn server(ws: &mut WebSocketServerOwned<NC, TcpStream>) {
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, &mut [b'H', b'e', b'l', b'l', b'o', b'!']))
      .await
      .unwrap();
    assert_eq!(
      ws.read_frame().await.unwrap().payload(),
      &mut [b'G', b'o', b'o', b'd', b'b', b'y', b'e', b'!']
    );
    ws.write_frame(&mut Frame::new_fin(OpCode::Close, &mut [])).await.unwrap();
  }
}

struct LargeFragmentedText;
impl<NC> Test<NC> for LargeFragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketClientOwned<NC, TcpStream>) {
    let bytes = || vec![b'1'; 256 * 1024];
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

  async fn server(ws: &mut WebSocketServerOwned<NC, TcpStream>) {
    let text = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!(&vec![b'1'; 10 * 256 * 1024], text.payload());
  }
}

struct PingAndText;
impl<NC> Test<NC> for PingAndText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketClientOwned<NC, TcpStream>) {
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [1, 2, 3])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, &mut [b'i', b'p', b'a', b't'])).await.unwrap();
    assert_eq!(OpCode::Pong, ws.read_frame().await.unwrap().op_code());
  }

  async fn server(ws: &mut WebSocketServerOwned<NC, TcpStream>) {
    assert_eq!(b"ipat", ws.read_frame().await.unwrap().payload());
  }
}

struct PingBetweenFragmentedText;
impl<NC> Test<NC> for PingBetweenFragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketClientOwned<NC, TcpStream>) {
    ws.write_frame(&mut Frame::new_unfin(OpCode::Text, &mut [b'1'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [b'9'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Continuation, &mut [b'2', b'3'])).await.unwrap();
    assert_eq!(OpCode::Pong, ws.read_frame().await.unwrap().op_code());
  }

  async fn server(ws: &mut WebSocketServerOwned<NC, TcpStream>) {
    let text = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!(&[b'1', b'2', b'3'], text.payload());
  }
}

struct SeveralBytes;
impl<NC> Test<NC> for SeveralBytes
where
  NC: NegotiatedCompression,
{
  async fn client(ws: &mut WebSocketClientOwned<NC, TcpStream>) {
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

  async fn server(ws: &mut WebSocketServerOwned<NC, TcpStream>) {
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
  async fn client(ws: &mut WebSocketClientOwned<NC, TcpStream>) {
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [b'0'])).await.unwrap();
    ws.write_frame(&mut Frame::new_fin(OpCode::Ping, &mut [b'1'])).await.unwrap();
    let _0 = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Pong, _0.op_code());
    assert_eq!(b"0", _0.payload());
    let _1 = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Pong, _1.op_code());
    assert_eq!(b"1", _1.payload());
    ws.write_frame(&mut Frame::new_fin(OpCode::Text, &mut [])).await.unwrap();
  }

  async fn server(ws: &mut WebSocketServerOwned<NC, TcpStream>) {
    let _0 = ws.read_frame().await.unwrap();
    assert_eq!(OpCode::Text, _0.op_code());
    assert_eq!(b"", _0.payload());
  }
}
