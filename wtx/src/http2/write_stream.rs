macro_rules! write_data_frames {
  (
    ($body:expr, $is_conn_open:expr, $send_params:expr, $stream:expr, $stream_id:expr),
    || $write_none_rslt:block,
    |$write_one:ident| $write_one_rslt:block,
    |$write_two:ident| $write_two_rslt:block
  ) => {{
    let mut iter = $body.chunks(*Usize::from($send_params.max_frame_len));
    loop {
      let should_stop = write_generic_frames!(
        iter,
        |bytes: &[u8]| DataFrame::new(u32::try_from(bytes.len()).unwrap_or_default(), $stream_id),
        |frame: &mut DataFrame| frame.set_eos(),
        || $write_none_rslt,
        |array| {
          let $write_one = array;
          $write_one_rslt;
        },
        |array| {
          let $write_two = array;
          $write_two_rslt;
        }
      );
      if should_stop {
        break;
      }
    }
  }};
}

macro_rules! write_generic_frames {
  (
    $iter:expr,
    $frame_cb:expr,
    $end_cb:expr,
    || $write_none_rslt:block,
    |$write_one:ident| $write_one_rslt:block,
    |$write_two:ident| $write_two_rslt:block
  ) => {
    if let Some(first) = $iter.next() {
      let mut first_frame = $frame_cb(first);
      let mut first_init_buffer = [0; 9];
      if let Some(second) = $iter.next() {
        adjust_frame_init(first, first_frame.bytes(), &mut first_init_buffer);
        let mut second_frame = $frame_cb(second);
        let mut second_init_buffer = [0; 9];
        if $iter.len() == 0 {
          $end_cb(&mut second_frame);
          adjust_frame_init(second, second_frame.bytes(), &mut second_init_buffer);
          let $write_two = (first_init_buffer, first, second_init_buffer, second);
          $write_two_rslt;
          true
        } else {
          adjust_frame_init(second, second_frame.bytes(), &mut second_init_buffer);
          let $write_two = (first_init_buffer, first, second_init_buffer, second);
          $write_two_rslt;
          false
        }
      } else {
        $end_cb(&mut first_frame);
        adjust_frame_init(first, first_frame.bytes(), &mut first_init_buffer);
        let $write_one = (first_init_buffer, first);
        $write_one_rslt;
        true
      }
    } else {
      $write_none_rslt;
      true
    }
  };
}

use crate::{
  http::Headers,
  http2::{
    misc::write_bytes, send_params::SendParams, ContinuationFrame, DataFrame, HeadersFrame,
    HpackEncoder, HpackStaticRequestHeaders, HpackStaticResponseHeaders, StreamState, U31,
  },
  misc::{AsyncBounds, ByteVector, Stream, Usize},
};

#[inline]
pub(crate) async fn write_stream<S, const IS_CLIENT: bool>(
  body: &[u8],
  headers: &Headers,
  hpack_enc: &mut HpackEncoder,
  hpack_enc_buffer: &mut ByteVector,
  (hsreqh, hsresh): (HpackStaticRequestHeaders<'_>, HpackStaticResponseHeaders),
  is_conn_open: bool,
  send_params: &SendParams,
  stream: &mut S,
  stream_id: U31,
  stream_state: &mut StreamState,
) -> crate::Result<()>
where
  S: AsyncBounds + Stream,
{
  let can_start_sending = if IS_CLIENT {
    matches!(stream_state, StreamState::Idle | StreamState::HalfClosedLocal)
  } else {
    matches!(stream_state, StreamState::Idle | StreamState::HalfClosedRemote)
  };
  if !can_start_sending {
    return Ok(());
  }

  if headers.bytes_len() > *Usize::from(send_params.max_expanded_headers_len) {
    return Err(crate::Error::VeryLargeHeadersLen);
  }

  hpack_enc_buffer.clear();
  if IS_CLIENT {
    hpack_enc.encode(hpack_enc_buffer, hsreqh.iter(), headers.iter())?;
  } else {
    hpack_enc.encode(hpack_enc_buffer, hsresh.iter(), headers.iter())?;
  }

  let hf = &mut HeadersFrame::new((hsreqh, hsresh), stream_id);
  if body.is_empty() {
    hf.set_eos();
  }

  if hpack_enc_buffer.is_empty() {
    write_single_header(body, hf, &[], is_conn_open, send_params, stream, stream_id).await?;
    *stream_state = StreamState::Open;
    return Ok(());
  }

  let mut iter = hpack_enc_buffer.chunks(*Usize::from(send_params.max_frame_len));

  if let Some(first) = iter.next() {
    write_single_header(body, hf, first, is_conn_open, send_params, stream, stream_id).await?;
    *stream_state = StreamState::Open;
  } else {
    return Ok(());
  }

  for _ in 0.._max_continuation_frames!() {
    let should_stop = write_generic_frames!(
      iter,
      |_| ContinuationFrame::new(stream_id),
      |frame: &mut ContinuationFrame| frame.set_eoh(),
      || {},
      |header_array| {
        let (a, b) = header_array;
        write_data_frames!(
          (body, is_conn_open, send_params, stream, stream_id),
          || {
            write_bytes([&a, b], is_conn_open, stream).await?;
          },
          |data_array| {
            let (c, d) = data_array;
            write_bytes([&a, b, &c, d], is_conn_open, stream).await?;
          },
          |data_array| {
            let (c, d, e, f) = data_array;
            write_bytes([&a, b, &c, d, &e, f], is_conn_open, stream).await?;
          }
        );
      },
      |header_array| {
        let (a, b, c, d) = header_array;
        write_data_frames!(
          (body, is_conn_open, send_params, stream, stream_id),
          || {
            write_bytes([&a, b], is_conn_open, stream).await?;
          },
          |data_array| {
            let (e, f) = data_array;
            write_bytes([&a, b, &c, d, &e, f], is_conn_open, stream).await?;
          },
          |data_array| {
            let (e, f, g, h) = data_array;
            write_bytes([&a, b, &c, d, &e, f, &g, h], is_conn_open, stream).await?;
          }
        );
      }
    );
    if should_stop {
      return Ok(());
    }
  }

  Ok(())
}

#[inline]
fn adjust_frame_init(content: &[u8], frame_init: [u8; 9], frame_init_buffer: &mut [u8; 9]) {
  let [a, b, c, d, e, f, g, h, i] = frame_init_buffer;
  let [_, j, k, l] = u32::try_from(content.len()).unwrap_or_default().to_be_bytes();
  let [_, _, _, m, n, o, p, q, r] = frame_init;
  *a = j;
  *b = k;
  *c = l;
  *d = m;
  *e = n;
  *f = o;
  *g = p;
  *h = q;
  *i = r;
}

#[inline]
async fn write_single_header<S>(
  body: &[u8],
  hf: &mut HeadersFrame<'_>,
  hf_content: &[u8],
  is_conn_open: bool,
  send_params: &SendParams,
  stream: &mut S,
  stream_id: U31,
) -> crate::Result<()>
where
  S: Stream,
{
  let init_buffer = &mut [0; 9];
  hf.set_eoh();
  adjust_frame_init(hf_content, hf.bytes(), init_buffer);

  write_data_frames!(
    (body, is_conn_open, send_params, stream, stream_id),
    || {
      write_bytes([init_buffer, hf_content], is_conn_open, stream).await?;
    },
    |data_array| {
      let (a, b) = data_array;
      write_bytes([init_buffer, hf_content, &a, b], is_conn_open, stream).await?;
    },
    |data_array| {
      let (a, b, c, d) = data_array;
      write_bytes([init_buffer, hf_content, &a, b, &c, d], is_conn_open, stream).await?;
    }
  );
  return Ok(());
}

#[cfg(test)]
mod tests {
  use crate::{
    http::{Headers, Method, Request, Response, ResponseData, StatusCode},
    http2::{ErrorCode, Http2Buffer, Http2Params, Http2Tokio, ReadFrameRslt, ReqResBuffer},
    misc::{UriString, Vector, _uri},
    rng::StaticRng,
  };
  use core::time::Duration;
  use tokio::net::{TcpListener, TcpStream};

  #[tokio::test]
  async fn streams_with_different_frames() {
    #[cfg(feature = "_tracing-subscriber")]
    let _rslt = crate::misc::tracing_subscriber_init();
    let uri = _uri();
    server(&uri).await;
    client(uri).await;
  }

  async fn client(uri: UriString) {
    let mut client = Http2Tokio::connect(
      Http2Buffer::new(StaticRng::default()),
      Http2Params::default(),
      TcpStream::connect(uri.host()).await.unwrap(),
    )
    .await
    .unwrap();
    let mut rrb = ReqResBuffer::default();
    rrb.data.reserve(3);
    rrb.headers.reserve(6, 1);

    let res = stream_client(&mut client, &mut rrb).await;
    _0(res.data.body(), res.data.headers());

    rrb.clear();
    rrb.headers.push_front(b"123", b"456", false).unwrap();
    let res = stream_client(&mut client, &mut rrb).await;
    _1(res.data.body(), res.data.headers());

    rrb.clear();
    rrb.data.extend_from_slice(b"123").unwrap();
    let res = stream_client(&mut client, &mut rrb).await;
    _2(res.data.body(), res.data.headers());

    rrb.clear();
    rrb.data.extend_from_slice(b"123").unwrap();
    rrb.headers.push_front(b"123", b"456", false).unwrap();
    let res = stream_client(&mut client, &mut rrb).await;
    _3(res.data.body(), res.data.headers());

    tokio::time::sleep(Duration::from_millis(2000)).await;
  }

  async fn server(uri: &UriString) {
    let listener = TcpListener::bind(uri.host()).await.unwrap();
    let _server_jh = tokio::spawn(async move {
      let (stream, _) = listener.accept().await.unwrap();
      let mut server =
        Http2Tokio::accept(Http2Buffer::new(StaticRng::default()), Http2Params::default(), stream)
          .await
          .unwrap();
      let mut rrb = ReqResBuffer::default();

      stream_server(&mut rrb, &mut server, |req| {
        _0(req.data, req.headers);
      })
      .await;
      stream_server(&mut rrb, &mut server, |req| {
        _1(req.data, req.headers);
      })
      .await;
      stream_server(&mut rrb, &mut server, |req| {
        _2(req.data, req.headers);
      })
      .await;
      stream_server(&mut rrb, &mut server, |req| {
        _3(req.data, req.headers);
      })
      .await;

      server.send_go_away(ErrorCode::NoError).await.unwrap();
    });
  }

  async fn stream_server<'rrb>(
    rrb: &'rrb mut ReqResBuffer,
    server: &mut Http2Tokio<Http2Buffer, TcpStream, false>,
    mut cb: impl FnMut(&Request<&mut Vector<u8>, &mut Headers, &str>),
  ) {
    loop {
      let rfr = server.stream(rrb).await.unwrap();
      let mut stream = match rfr {
        ReadFrameRslt::ClosedConnection | ReadFrameRslt::ClosedStream => {
          panic!();
        }
        ReadFrameRslt::IdleConnection => {
          continue;
        }
        ReadFrameRslt::Resource(elem) => elem,
      };
      let req = stream.recv_req(rrb).await.unwrap().resource().unwrap();
      cb(&req);
      stream.send_res(Response::http2((&req.data, &req.headers), StatusCode::Ok)).await.unwrap();
      break;
    }
  }

  async fn stream_client<'rrb>(
    client: &mut Http2Tokio<Http2Buffer, TcpStream, true>,
    rrb: &'rrb mut ReqResBuffer,
  ) -> Response<&'rrb mut ReqResBuffer> {
    let mut stream = client.stream().await.unwrap();
    stream.send_req(rrb.as_http2_request_ref(Method::Get)).await.unwrap();
    stream.recv_res(rrb).await.unwrap().resource().unwrap()
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
}
