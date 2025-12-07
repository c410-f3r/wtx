use crate::{
  executor::Runtime,
  http::{Header, Headers, Method, ReqResBuffer, ReqResData, Request, StatusCode},
  http2::{Http2, Http2Buffer, Http2ErrorCode, Http2Params},
  misc::{UriRef, UriString},
  rng::{Xorshift64, simple_seed},
  tests::_uri,
};
use core::time::Duration;
use std::net::{TcpListener, TcpStream};

// FIXME(MIRI): socket support
#[cfg_attr(miri, ignore)]
#[test]
fn connections() {
  let runtime = Runtime::new();
  let uri = _uri();
  server(&uri, &runtime);
  let client_fut = client(&uri, &runtime);
  runtime.block_on(client_fut).unwrap();
}

async fn client(uri: &UriString, runtime: &Runtime) {
  let mut rrb = ReqResBuffer::empty();
  rrb.headers.reserve(6, 1).unwrap();
  let stream = TcpStream::connect(uri.hostname_with_implied_port()).unwrap();
  let (frame_header, mut http2) = Http2::connect(
    Http2Buffer::new(&mut Xorshift64::from(simple_seed())),
    Http2Params::default(),
    (stream.try_clone().unwrap(), stream),
  )
  .await
  .unwrap();
  let _jh = runtime.spawn_threaded(frame_header).unwrap();

  let uri_ref = uri.to_ref();

  rrb = stream_client(&mut http2, rrb, &uri_ref).await;
  _0(rrb.body(), rrb.headers());

  rrb.clear();
  rrb.headers.push_from_iter(Header::from_name_and_value("123", ["456"])).unwrap();
  rrb = stream_client(&mut http2, rrb, &uri_ref).await;
  _1(rrb.body(), rrb.headers());

  rrb.clear();
  rrb.body.extend_from_copyable_slice(b"123").unwrap();
  rrb = stream_client(&mut http2, rrb, &uri_ref).await;
  _2(rrb.body(), rrb.headers());

  rrb.clear();
  rrb.body.extend_from_copyable_slice(b"123").unwrap();
  rrb.headers.push_from_iter(Header::from_name_and_value("123", ["456"])).unwrap();
  rrb = stream_client(&mut http2, rrb, &uri_ref).await;
  _3(rrb.body(), rrb.headers());

  http2.send_go_away(Http2ErrorCode::NoError).await;

  crate::misc::sleep(Duration::from_millis(100)).await.unwrap();
}

fn server(uri: &UriString, runtime: &Runtime) {
  let listener = TcpListener::bind(uri.hostname_with_implied_port()).unwrap();
  let runtime_fut = runtime.clone();
  let _server_jh = runtime
    .spawn_threaded(async move {
      let (stream, _) = listener.accept().unwrap();
      let mut rrb = ReqResBuffer::empty();
      let (frame_header, mut http2) = Http2::accept(
        Http2Buffer::new(&mut Xorshift64::from(simple_seed())),
        Http2Params::default(),
        (stream.try_clone().unwrap(), stream),
      )
      .await
      .unwrap();
      let _jh = runtime_fut.spawn_threaded(frame_header);

      rrb = stream_server(&mut http2, rrb, |req| {
        _0(req.rrd.body(), req.rrd.headers());
      })
      .await;
      rrb = stream_server(&mut http2, rrb, |req| {
        _1(req.rrd.body(), req.rrd.headers());
      })
      .await;
      rrb = stream_server(&mut http2, rrb, |req| {
        _2(req.rrd.body(), req.rrd.headers());
      })
      .await;
      let _rrb = stream_server(&mut http2, rrb, |req| {
        _3(req.rrd.body(), req.rrd.headers());
      })
      .await;
    })
    .unwrap();
}

async fn stream_server(
  server: &mut Http2<Http2Buffer, TcpStream, false>,
  rrb: ReqResBuffer,
  mut cb: impl FnMut(Request<&mut ReqResBuffer>),
) -> ReqResBuffer {
  let (mut stream, _) = server.stream(rrb, |_, _| {}).await.unwrap().unwrap();
  let (_, mut req_rrb) = stream.recv_req().await.unwrap();
  cb(req_rrb.as_http2_request_mut(stream.method()));
  let _ = stream.send_res(req_rrb.as_http2_response(StatusCode::Ok)).await.unwrap();
  req_rrb
}

async fn stream_client(
  client: &mut Http2<Http2Buffer, TcpStream, true>,
  rrb: ReqResBuffer,
  uri: &UriRef<'_>,
) -> ReqResBuffer {
  let mut stream = client.stream().await.unwrap();
  let _ = stream.send_req(rrb.as_http2_request(Method::Get), uri).await.unwrap();
  stream.recv_res(rrb).await.unwrap().1
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
