macro_rules! call_tests {
  (($ty:ident, $fb:expr, $ws:expr), $($struct:ident),+ $(,)?) => {
    $(
      $struct::$ty($fb, $ws).await;
      tokio::time::sleep(Duration::from_millis(200)).await;
    )+
  };
}

use crate::{
  misc::NoStdRng,
  tests::_uri,
  web_socket::{
    compression::NegotiatedCompression, frame::FrameMutVec, handshake::HeadersBuffer, Compression,
    FrameBufferVec, OpCode, WebSocket, WebSocketBuffer, WebSocketClient, WebSocketClientOwned,
    WebSocketServer, WebSocketServerOwned,
  },
};
use alloc::vec;
use core::{
  sync::atomic::{AtomicBool, Ordering},
  time::Duration,
};
use tokio::net::{TcpListener, TcpStream};

static HAS_SERVER_FINISHED: AtomicBool = AtomicBool::new(false);

#[tokio::test]
async fn client_and_server_frames() {
  #[cfg(feature = "_tracing-tree")]
  let _rslt = crate::misc::tracing_tree_init(None);
  do_test_client_and_server_frames((), ()).await;
}

#[cfg(feature = "flate2")]
#[tokio::test]
async fn client_and_server_frames_compression() {
  use crate::web_socket::compression::Flate2;
  #[cfg(feature = "_tracing-tree")]
  let _rslt = crate::misc::tracing_tree_init(None);
  do_test_client_and_server_frames((), Flate2::default()).await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  do_test_client_and_server_frames(Flate2::default(), ()).await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  do_test_client_and_server_frames(Flate2::default(), Flate2::default()).await;
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

  let listener = TcpListener::bind(uri.host()).await.unwrap();
  let _server_jh = tokio::spawn(async move {
    let (stream, _) = listener.accept().await.unwrap();
    let mut fb = FrameBufferVec::with_capacity(0);
    let mut ws = WebSocketServer::accept(
      server_compression,
      NoStdRng::default(),
      stream,
      WebSocketBuffer::with_capacity(0, 0),
      |_| true,
    )
    .await
    .unwrap();
    call_tests!(
      (server, &mut fb, &mut ws),
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

  let mut fb = FrameBufferVec::with_capacity(0);
  let (_, mut ws) = WebSocketClient::connect(
    client_compression,
    &mut fb,
    [],
    &mut HeadersBuffer::default(),
    NoStdRng::default(),
    TcpStream::connect(uri.host()).await.unwrap(),
    &uri.to_ref(),
    WebSocketBuffer::with_capacity(0, 0),
  )
  .await
  .unwrap();
  call_tests!(
    (client, &mut fb, &mut ws),
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
  async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<NC, NoStdRng, TcpStream>);

  async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<NC, NoStdRng, TcpStream>);
}

struct FragmentedText;
impl<NC> Test<NC> for FragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<NC, NoStdRng, TcpStream>) {
    write(FrameMutVec::new_unfin(fb, OpCode::Text, b"1").unwrap(), ws).await;
    write(FrameMutVec::new_fin(fb, OpCode::Continuation, b"23").unwrap(), ws).await;
  }

  async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<NC, NoStdRng, TcpStream>) {
    let text = ws.read_frame(fb).await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!(b"123", text.fb().payload());
  }
}

struct HelloAndGoodbye;
impl<NC> Test<NC> for HelloAndGoodbye
where
  NC: NegotiatedCompression,
{
  async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<NC, NoStdRng, TcpStream>) {
    let hello = ws.read_frame(fb).await.unwrap();
    assert_eq!(OpCode::Text, hello.op_code());
    assert_eq!(b"Hello!", hello.fb().payload());
    write(FrameMutVec::new_fin(fb, OpCode::Text, b"Goodbye!").unwrap(), ws).await;
    assert_eq!(OpCode::Close, ws.read_frame(fb).await.unwrap().op_code());
  }

  async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<NC, NoStdRng, TcpStream>) {
    write(FrameMutVec::new_fin(fb, OpCode::Text, b"Hello!").unwrap(), ws).await;
    assert_eq!(ws.read_frame(&mut *fb).await.unwrap().fb().payload(), b"Goodbye!");
    write(FrameMutVec::new_fin(fb, OpCode::Close, &[]).unwrap(), ws).await;
  }
}

struct LargeFragmentedText;
impl<NC> Test<NC> for LargeFragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<NC, NoStdRng, TcpStream>) {
    let bytes = vec![51; 256 * 1024];
    write(FrameMutVec::new_unfin(fb, OpCode::Text, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
    write(FrameMutVec::new_fin(fb, OpCode::Continuation, &bytes).unwrap(), ws).await;
  }

  async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<NC, NoStdRng, TcpStream>) {
    let text = ws.read_frame(fb).await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!(&vec![51; 10 * 256 * 1024], text.fb().payload());
  }
}

struct PingAndText;
impl<NC> Test<NC> for PingAndText
where
  NC: NegotiatedCompression,
{
  async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<NC, NoStdRng, TcpStream>) {
    write(FrameMutVec::new_fin(fb, OpCode::Ping, b"123").unwrap(), ws).await;
    write(FrameMutVec::new_fin(fb, OpCode::Text, b"ipat").unwrap(), ws).await;
    assert_eq!(OpCode::Pong, ws.read_frame(fb).await.unwrap().op_code());
  }

  async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<NC, NoStdRng, TcpStream>) {
    assert_eq!(b"ipat", ws.read_frame(fb).await.unwrap().fb().payload());
  }
}

struct PingBetweenFragmentedText;
impl<NC> Test<NC> for PingBetweenFragmentedText
where
  NC: NegotiatedCompression,
{
  async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<NC, NoStdRng, TcpStream>) {
    write(FrameMutVec::new_unfin(fb, OpCode::Text, b"1").unwrap(), ws).await;
    write(FrameMutVec::new_fin(fb, OpCode::Ping, b"9").unwrap(), ws).await;
    write(FrameMutVec::new_fin(fb, OpCode::Continuation, b"23").unwrap(), ws).await;
    assert_eq!(OpCode::Pong, ws.read_frame(fb).await.unwrap().op_code());
  }

  async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<NC, NoStdRng, TcpStream>) {
    assert_eq!(OpCode::Text, ws.read_frame(fb).await.unwrap().op_code());
  }
}

struct SeveralBytes;
impl<NC> Test<NC> for SeveralBytes
where
  NC: NegotiatedCompression,
{
  async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<NC, NoStdRng, TcpStream>) {
    write(FrameMutVec::new_unfin(fb, OpCode::Text, &[206]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[186]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[225]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[189]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[185]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[207]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[131]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[206]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[188]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[206]).unwrap(), ws).await;
    write(FrameMutVec::new_unfin(fb, OpCode::Continuation, &[181]).unwrap(), ws).await;
    write(FrameMutVec::new_fin(fb, OpCode::Continuation, &[]).unwrap(), ws).await;
  }

  async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<NC, NoStdRng, TcpStream>) {
    let text = ws.read_frame(fb).await.unwrap();
    assert_eq!(OpCode::Text, text.op_code());
    assert_eq!("κόσμε".as_bytes(), text.fb().payload());
  }
}

struct TwoPings;
impl<NC> Test<NC> for TwoPings
where
  NC: NegotiatedCompression,
{
  async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<NC, NoStdRng, TcpStream>) {
    write(FrameMutVec::new_fin(fb, OpCode::Ping, b"0").unwrap(), ws).await;
    write(FrameMutVec::new_fin(fb, OpCode::Ping, b"1").unwrap(), ws).await;
    let _0 = ws.read_frame(fb).await.unwrap();
    assert_eq!(OpCode::Pong, _0.op_code());
    assert_eq!(b"0", _0.fb().payload());
    let _1 = ws.read_frame(fb).await.unwrap();
    assert_eq!(OpCode::Pong, _1.op_code());
    assert_eq!(b"1", _1.fb().payload());
    write(FrameMutVec::new_fin(fb, OpCode::Text, b"").unwrap(), ws).await;
  }

  async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<NC, NoStdRng, TcpStream>) {
    let _0 = ws.read_frame(fb).await.unwrap();
    assert_eq!(OpCode::Text, _0.op_code());
    assert_eq!(b"", _0.fb().payload());
  }
}

async fn write<NC, const IS_CLIENT: bool>(
  mut frame: FrameMutVec<'_, IS_CLIENT>,
  ws: &mut WebSocket<NC, NoStdRng, TcpStream, WebSocketBuffer, IS_CLIENT>,
) where
  NC: NegotiatedCompression,
{
  ws.write_frame(&mut frame).await.unwrap();
}
