macro_rules! manage_send {
  ($before_body:expr, $body:expr, $buffer:expr, $wp:expr) => {
    if !$wp.manage_send(U31::from_u32(u32::from_be_bytes([0, $buffer[1], $buffer[2], $buffer[3]])))
    {
      *$body = $before_body;
      return Ok(Http2RsltExt::Idle);
    }
  };
}

macro_rules! write_data_frames {
  (
    ($body:expr, $hps:expr, $is_conn_open:expr, $stream:expr, $stream_id:expr, $wp:expr),
    || $write_none_rslt:block,
    |$write_one:ident| $write_one_rslt:block,
    |$write_two:ident| $write_two_rslt:block
  ) => {{
    let mut iter = $body.chunks(*Usize::from($hps.max_frame_len));
    loop {
      let should_stop = write_generic_frames!(
        ($body, iter, $wp),
        |bytes: &[u8]| DataFrame::new(
          U31::from_u32(u32::try_from(bytes.len()).unwrap_or_default()),
          $stream_id,
        ),
        |frame: &mut DataFrame| frame.set_eos(),
        || $write_none_rslt,
        |tuple| {
          let $write_one = tuple;
          $write_one_rslt;
        },
        |tuple| {
          let $write_two = tuple;
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
    ($body:expr, $iter:expr, $wp:expr),
    $frame_cb:expr,
    $end_cb:expr,
    || $write_none_rslt:block,
    |$write_one:ident| $write_one_rslt:block,
    |$write_two:ident| $write_two_rslt:block
  ) => {{
    let before_first = *$body;
    if let Some(first) = $iter.next() {
      let mut first_frame = $frame_cb(first);
      let mut first_init_buffer = [0; 9];
      let before_second = *$body;
      if let Some(second) = $iter.next() {
        adjust_frame_init(first, first_frame.bytes(), &mut first_init_buffer);
        manage_send!(before_first, $body, first_init_buffer, $wp);
        let mut second_frame = $frame_cb(second);
        let mut second_init_buffer = [0; 9];
        if $iter.len() == 0 {
          $end_cb(&mut second_frame);
          adjust_frame_init(second, second_frame.bytes(), &mut second_init_buffer);
          manage_send!(before_second, $body, second_init_buffer, $wp);
          let $write_two = (first_init_buffer, first, second_init_buffer, second);
          $write_two_rslt;
          true
        } else {
          adjust_frame_init(second, second_frame.bytes(), &mut second_init_buffer);
          manage_send!(before_second, $body, second_init_buffer, $wp);
          let $write_two = (first_init_buffer, first, second_init_buffer, second);
          $write_two_rslt;
          false
        }
      } else {
        $end_cb(&mut first_frame);
        adjust_frame_init(first, first_frame.bytes(), &mut first_init_buffer);
        manage_send!(before_first, $body, first_init_buffer, $wp);
        let $write_one = (first_init_buffer, first);
        $write_one_rslt;
        true
      }
    } else {
      $write_none_rslt;
      true
    }
  }};
}

use crate::{
  http2::{
    http2_params_send::Http2ParamsSend,
    http2_rslt::Http2RsltExt,
    misc::{reset_stream, write_array},
    window::WindowsPair,
    ContinuationFrame, DataFrame, HeadersFrame, StreamState, U31,
  },
  misc::{AsyncBounds, ByteVector, Stream, Usize},
};

#[inline]
pub(crate) async fn write_stream<S, const IS_CLIENT: bool>(
  body: &mut &[u8],
  hf: &mut HeadersFrame<'_>,
  hpack_enc_buffer: &mut ByteVector,
  hps: &Http2ParamsSend,
  is_conn_open: bool,
  stream: &mut S,
  stream_state: &mut StreamState,
  streams_num: &mut u32,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<Http2RsltExt<()>>
where
  S: AsyncBounds + Stream,
{
  if wp.is_invalid_send() {
    return Ok(Http2RsltExt::Idle);
  }
  let can_start_sending = if IS_CLIENT {
    matches!(stream_state, StreamState::Idle | StreamState::HalfClosedLocal)
  } else {
    matches!(stream_state, StreamState::Idle | StreamState::HalfClosedRemote)
  };
  if !can_start_sending {
    return Err(crate::Error::InvalidStreamState);
  }

  let stream_id = hf.stream_id();

  if body.is_empty() {
    hf.set_eos();
  }

  if hpack_enc_buffer.is_empty() {
    hf.set_eoh();
    hre_resource_or_return!(
      write_init_header(body, hf, &[], hps, is_conn_open, stream, stream_id, wp).await?
    );
    *stream_state = StreamState::Open;
    return Ok(Http2RsltExt::Resource(()));
  }

  let mut iter = hpack_enc_buffer.chunks(*Usize::from(hps.max_frame_len));

  if let Some(first) = iter.next() {
    if iter.len() == 0 {
      hf.set_eoh();
    }
    hre_resource_or_return!(
      write_init_header(body, hf, first, hps, is_conn_open, stream, stream_id, wp).await?
    );
    *stream_state = StreamState::Open;
  } else {
    return Ok(Http2RsltExt::Resource(()));
  }

  for _ in 0.._max_continuation_frames!() {
    let should_stop = write_generic_frames!(
      (body, iter, wp),
      |_| ContinuationFrame::new(stream_id),
      |frame: &mut ContinuationFrame| frame.set_eoh(),
      || {},
      |header_array| {
        let (a, b) = header_array;
        write_data_frames!(
          (body, hps, is_conn_open, stream, stream_id, wp),
          || {
            write_array([&a, b], is_conn_open, stream).await?;
          },
          |data_array| {
            let (c, d) = data_array;
            write_array([&a, b, &c, d], is_conn_open, stream).await?;
          },
          |data_array| {
            let (c, d, e, f) = data_array;
            write_array([&a, b, &c, d, &e, f], is_conn_open, stream).await?;
          }
        );
      },
      |header_array| {
        let (a, b, c, d) = header_array;
        write_data_frames!(
          (body, hps, is_conn_open, stream, stream_id, wp),
          || {
            write_array([&a, b], is_conn_open, stream).await?;
          },
          |data_array| {
            let (e, f) = data_array;
            write_array([&a, b, &c, d, &e, f], is_conn_open, stream).await?;
          },
          |data_array| {
            let (e, f, g, h) = data_array;
            write_array([&a, b, &c, d, &e, f, &g, h], is_conn_open, stream).await?;
          }
        );
      }
    );
    if should_stop {
      return Ok(Http2RsltExt::Resource(()));
    }
  }

  Ok(reset_stream(stream_state, streams_num))
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
async fn write_init_header<S>(
  body: &mut &[u8],
  hf: &mut HeadersFrame<'_>,
  hf_content: &[u8],
  hps: &Http2ParamsSend,
  is_conn_open: bool,
  stream: &mut S,
  stream_id: U31,
  wp: &mut WindowsPair<'_>,
) -> crate::Result<Http2RsltExt<()>>
where
  S: Stream,
{
  let init_buffer = &mut [0; 9];
  adjust_frame_init(hf_content, hf.bytes(), init_buffer);

  write_data_frames!(
    (body, hps, is_conn_open, stream, stream_id, wp),
    || {
      write_array([init_buffer, hf_content], is_conn_open, stream).await?;
    },
    |data_array| {
      let (a, b) = data_array;
      write_array([init_buffer, hf_content, &a, b], is_conn_open, stream).await?;
    },
    |data_array| {
      let (a, b, c, d) = data_array;
      write_array([init_buffer, hf_content, &a, b, &c, d], is_conn_open, stream).await?;
    }
  );
  return Ok(Http2RsltExt::Resource(()));
}

#[cfg(test)]
mod tests {
  use crate::{
    http::{Headers, Method, ReqResData, RequestStr, Response, StatusCode},
    http2::{ErrorCode, Http2Buffer, Http2Params, Http2Tokio, ReqResBuffer},
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
    let mut rrb = ReqResBuffer::with_capacity(3, 6, 1, 6);

    let res = stream_client(&mut client, &mut rrb).await;
    _0(res.data.body(), res.data.headers());

    rrb.clear();
    rrb.headers.push_front(b"123", b"456", false).unwrap();
    let res = stream_client(&mut client, &mut rrb).await;
    _1(res.data.body(), res.data.headers());

    rrb.clear();
    rrb.body.extend_from_slice(b"123").unwrap();
    let res = stream_client(&mut client, &mut rrb).await;
    _2(res.data.body(), res.data.headers());

    rrb.clear();
    rrb.body.extend_from_slice(b"123").unwrap();
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
        _0(req.data.body(), req.data.headers());
      })
      .await;
      stream_server(&mut rrb, &mut server, |req| {
        _1(req.data.body(), req.data.headers());
      })
      .await;
      stream_server(&mut rrb, &mut server, |req| {
        _2(req.data.body(), req.data.headers());
      })
      .await;
      stream_server(&mut rrb, &mut server, |req| {
        _3(req.data.body(), req.data.headers());
      })
      .await;

      server.send_go_away(ErrorCode::NoError).await.unwrap();
    });
  }

  async fn stream_server<'rrb>(
    rrb: &'rrb mut ReqResBuffer,
    server: &mut Http2Tokio<Http2Buffer, TcpStream, false>,
    mut cb: impl FnMut(&RequestStr<'_, (&mut Vector<u8>, &mut Headers)>),
  ) {
    loop {
      let mut stream = server.stream(rrb).await.unwrap().resource().unwrap();
      let req = stream.recv_req(rrb).await.unwrap().resource().unwrap();
      cb(&req);
      stream
        .send_res(Response::http2(&req.data, StatusCode::Ok))
        .await
        .unwrap()
        .resource()
        .unwrap();
      break;
    }
  }

  async fn stream_client<'rrb>(
    client: &mut Http2Tokio<Http2Buffer, TcpStream, true>,
    rrb: &'rrb mut ReqResBuffer,
  ) -> Response<&'rrb mut ReqResBuffer> {
    let mut stream = client.stream().await.unwrap();
    stream.send_req(rrb.as_http2_request(Method::Get)).await.unwrap().resource().unwrap();
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
