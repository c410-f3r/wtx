use crate::{
  collections::Vector,
  executor::StdRuntime,
  http::{
    Header, Headers, HttpRecvParams, MsgBufferString, MsgData, MsgDataMut, ReqBuilder, Request,
    StatusCode,
  },
  http2::{Http2, Http2Buffer, Http2ErrorCode},
  misc::{UriRef, UriString},
  rng::{ChaCha20, CryptoSeedableRng, SeedableRng, Xorshift64},
  stream::Stream as _,
  tests::_uri,
  tls::{TlsAcceptor, TlsConfig, TlsConnector, TlsModePlainText},
};
use core::time::Duration;
use std::net::{TcpListener, TcpStream};

// FIXME(MIRI): socket support
#[cfg_attr(miri, ignore)]
#[test]
fn connections() {
  let runtime = StdRuntime::new();
  let uri = _uri();
  server(&uri, &runtime);
  let client_fut = client(&uri, &runtime);
  runtime.block_on(client_fut);
}

async fn client(uri: &UriString, runtime: &StdRuntime) {
  let mut msg_buffer = MsgBufferString::default();
  msg_buffer.headers.reserve(6, 1).unwrap();
  let stream = TcpStream::connect(uri.hostname_with_implied_port()).unwrap();
  let tls_stream =
    TlsConnector::new(&TlsConfig::plaintext(), ChaCha20::from_std_random().unwrap(), stream)
      .connect()
      .await
      .unwrap()
      .tls_stream;
  let (frame_header, mut http2) = Http2::connect(
    Http2Buffer::new(&mut Xorshift64::from_simple_seed().unwrap()),
    HttpRecvParams::with_optioned_params(),
    tls_stream.into_split().unwrap(),
  )
  .await
  .unwrap();
  let _jh = runtime.spawn(frame_header).unwrap();

  let uri_ref = uri.to_ref();

  msg_buffer = stream_client(&mut http2, msg_buffer, uri_ref).await;
  _0(msg_buffer.body(), msg_buffer.headers());

  msg_buffer.clear();
  msg_buffer.headers.push_from_iter(Header::from_name_and_value("123", ["456"])).unwrap();
  msg_buffer = stream_client(&mut http2, msg_buffer, uri_ref).await;
  _1(msg_buffer.body(), msg_buffer.headers());

  msg_buffer.clear();
  msg_buffer.body.extend_from_copyable_slice(b"123").unwrap();
  msg_buffer = stream_client(&mut http2, msg_buffer, uri_ref).await;
  _2(msg_buffer.body(), msg_buffer.headers());

  msg_buffer.clear();
  msg_buffer.body.extend_from_copyable_slice(b"123").unwrap();
  msg_buffer.headers.push_from_iter(Header::from_name_and_value("123", ["456"])).unwrap();
  msg_buffer = stream_client(&mut http2, msg_buffer, uri_ref).await;
  _3(msg_buffer.body(), msg_buffer.headers());

  http2.send_go_away(Http2ErrorCode::NoError).await;

  crate::futures::Sleep::new(Duration::from_millis(100)).unwrap().await.unwrap();
}

fn server(uri: &UriString, runtime: &StdRuntime) {
  let listener = TcpListener::bind(uri.hostname_with_implied_port()).unwrap();
  let runtime_fut = runtime.clone();
  let _server_jh = runtime
    .spawn(async move {
      let (stream, _) = listener.accept().unwrap();
      let tls_stream =
        TlsAcceptor::new(&TlsConfig::plaintext(), ChaCha20::from_std_random().unwrap(), stream)
          .accept()
          .await
          .unwrap()
          .tls_stream;
      let (frame_header, mut http2) = Http2::accept(
        Http2Buffer::new(&mut Xorshift64::from_simple_seed().unwrap()),
        HttpRecvParams::with_optioned_params(),
        tls_stream.into_split().unwrap(),
      )
      .await
      .unwrap();
      let _jh = runtime_fut.spawn(frame_header);
      stream_server(&mut http2, |req| {
        _0(req.msg_data.body(), req.msg_data.headers());
      })
      .await;
      stream_server(&mut http2, |req| {
        _1(req.msg_data.body(), req.msg_data.headers());
      })
      .await;
      stream_server(&mut http2, |req| {
        _2(req.msg_data.body(), req.msg_data.headers());
      })
      .await;
      stream_server(&mut http2, |req| {
        _3(req.msg_data.body(), req.msg_data.headers());
      })
      .await;
    })
    .unwrap();
}

async fn stream_server(
  server: &mut Http2<TcpStream, TlsModePlainText, false>,
  mut cb: impl FnMut(Request<&mut MsgBufferString>),
) {
  let (mut stream, _) = server.stream(|_, _| {}).await.unwrap().unwrap();
  let (_, mut req_rrb) = stream.recv_req().await.unwrap();
  cb(req_rrb.as_http2_request_mut(stream.method()));
  let _ =
    stream.send_res(&mut Vector::new(), req_rrb.as_http2_response(StatusCode::Ok)).await.unwrap();
}

async fn stream_client(
  client: &mut Http2<TcpStream, TlsModePlainText, true>,
  msg_buffer: MsgBufferString,
  uri: UriRef<'_>,
) -> MsgBufferString {
  let mut stream = client.stream().await.unwrap();
  let rb = ReqBuilder::get((msg_buffer.body.as_ref(), &msg_buffer.headers, uri));
  let _ = stream.send_req(&mut Vector::new(), rb.into_request()).await.unwrap();
  stream.recv_res().await.unwrap().1
}

#[track_caller]
fn _0(body: &[u8], headers: &Headers) {
  assert_eq!((body.len(), headers.bytes_len(), headers.headers_len()), (0, 0, 0));
}
#[track_caller]
fn _1(body: &[u8], headers: &Headers) {
  assert_eq!((body.len(), headers.bytes_len(), headers.headers_len()), (0, 6, 1));
}
#[track_caller]
fn _2(body: &[u8], headers: &Headers) {
  assert_eq!((body.len(), headers.bytes_len(), headers.headers_len()), (3, 0, 0));
}
#[track_caller]
fn _3(body: &[u8], headers: &Headers) {
  assert_eq!((body.len(), headers.bytes_len(), headers.headers_len()), (3, 6, 1));
}
