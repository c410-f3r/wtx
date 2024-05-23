use crate::{
  http::{Headers, Method, ReqResData, RequestStr, StatusCode},
  http2::{Http2Buffer, Http2ErrorCode, Http2Params, Http2Tokio, StreamBuffer},
  misc::{UriString, Vector, _uri},
  rng::StaticRng,
};
use alloc::boxed::Box;
use core::time::Duration;
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn connections() {
  #[cfg(feature = "_tracing-subscriber")]
  let _rslt = crate::misc::tracing_subscriber_init();
  let uri = _uri();
  server(&uri).await;
  client(uri).await;
}

async fn client(uri: UriString) {
  let mut sb = Box::new(StreamBuffer::default());
  sb.rrb.body.reserve(3);
  sb.rrb.headers.set_max_bytes(6);
  sb.rrb.headers.reserve(6, 1);
  let mut client = Http2Tokio::connect(
    Http2Buffer::new(StaticRng::default()),
    Http2Params::default(),
    TcpStream::connect(uri.host()).await.unwrap(),
  )
  .await
  .unwrap();

  sb = stream_client(&mut client, sb).await;
  _0(&sb.rrb.body, &sb.rrb.headers);

  sb.clear();
  sb.rrb.headers.push_front((b"123", b"456").into()).unwrap();
  sb = stream_client(&mut client, sb).await;
  _1(&sb.rrb.body, &sb.rrb.headers);

  sb.clear();
  sb.rrb.body.extend_from_slice(b"123").unwrap();
  sb = stream_client(&mut client, sb).await;
  _2(&sb.rrb.body, &sb.rrb.headers);

  sb.clear();
  sb.rrb.body.extend_from_slice(b"123").unwrap();
  sb.rrb.headers.push_front((b"123", b"456").into()).unwrap();
  sb = stream_client(&mut client, sb).await;
  _3(&sb.rrb.body, &sb.rrb.headers);

  client.send_go_away(Http2ErrorCode::NoError).await;

  tokio::time::sleep(Duration::from_millis(100)).await;
}

async fn server(uri: &UriString) {
  let listener = TcpListener::bind(uri.host()).await.unwrap();
  let _server_jh = tokio::spawn(async move {
    let (stream, _) = listener.accept().await.unwrap();
    let mut sb = Box::new(StreamBuffer::default());
    let mut server =
      Http2Tokio::accept(Http2Buffer::new(StaticRng::default()), Http2Params::default(), stream)
        .await
        .unwrap();

    sb = stream_server(&mut server, sb, |req| {
      _0(req.data.body(), req.data.headers());
    })
    .await;
    sb = stream_server(&mut server, sb, |req| {
      _1(req.data.body(), req.data.headers());
    })
    .await;
    sb = stream_server(&mut server, sb, |req| {
      _2(req.data.body(), req.data.headers());
    })
    .await;
    let _sb = stream_server(&mut server, sb, |req| {
      _3(req.data.body(), req.data.headers());
    })
    .await;
  });
}

async fn stream_server(
  server: &mut Http2Tokio<Http2Buffer<Box<StreamBuffer>>, TcpStream, Box<StreamBuffer>, false>,
  sb: Box<StreamBuffer>,
  mut cb: impl FnMut(RequestStr<'_, (&mut Vector<u8>, &mut Headers)>),
) -> Box<StreamBuffer> {
  loop {
    let mut stream = server.stream(sb).await.unwrap();
    let (mut req_sb, method) = stream.recv_req().await.unwrap();
    cb(req_sb.rrb.as_http2_request_mut(method));
    stream
      .send_res(&mut req_sb.hpack_enc_buffer, req_sb.rrb.as_http2_response(StatusCode::Ok))
      .await
      .unwrap();
    break req_sb;
  }
}

async fn stream_client(
  client: &mut Http2Tokio<Http2Buffer<Box<StreamBuffer>>, TcpStream, Box<StreamBuffer>, true>,
  mut sb: Box<StreamBuffer>,
) -> Box<StreamBuffer> {
  let mut stream = client.stream().await.unwrap();
  stream.send_req(&mut sb.hpack_enc_buffer, sb.rrb.as_http2_request(Method::Get)).await.unwrap();
  stream.recv_res(sb).await.unwrap().0
}

#[track_caller]
fn _0(data: &[u8], headers: &Headers) {
  assert_eq!((data.len(), headers.bytes_len(), headers.elements_len()), (0, 0, 0));
}
#[track_caller]
fn _1(data: &[u8], headers: &Headers) {
  assert_eq!((data.len(), headers.bytes_len(), headers.elements_len()), (0, 6, 1));
}
#[track_caller]
fn _2(data: &[u8], headers: &Headers) {
  assert_eq!((data.len(), headers.bytes_len(), headers.elements_len()), (3, 0, 0));
}
#[track_caller]
fn _3(data: &[u8], headers: &Headers) {
  assert_eq!((data.len(), headers.bytes_len(), headers.elements_len()), (3, 6, 1));
}
