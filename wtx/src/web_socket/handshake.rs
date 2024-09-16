#[cfg(test)]
mod tests;

use crate::{
  http::{GenericHeader as _, GenericRequest as _, HttpError, KnownHeaderName, Method},
  misc::{bytes_split1, FilledBufferWriter, LeaseMut, Rng, Stream, UriRef, VectorError},
  web_socket::{
    compression::NegotiatedCompression, misc::_trim_bytes, Compression, FrameBufferVec,
    WebSocketBuffer, WebSocketClient, WebSocketError, WebSocketServer,
  },
};
use base64::{engine::general_purpose::STANDARD, Engine};
use httparse::{Header, Request, Response, Status, EMPTY_HEADER};
use sha1::{Digest, Sha1};

const MAX_READ_LEN: usize = 2 * 1024;
const MAX_READ_HEADER_LEN: usize = 64;

/// Necessary to decode incoming bytes of responses or requests.
#[derive(Debug)]
pub struct HeadersBuffer<H, const N: usize> {
  pub(crate) _headers: [H; N],
}

impl<const N: usize> Default for HeadersBuffer<Header<'_>, N> {
  #[inline]
  fn default() -> Self {
    Self { _headers: [EMPTY_HEADER; N] }
  }
}

impl<NC, RNG, S, WSB> WebSocketServer<NC, RNG, S, WSB>
where
  NC: NegotiatedCompression,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Reads external data to establish an WebSocket connection.
  #[inline]
  pub async fn accept<C>(
    compression: C,
    rng: RNG,
    mut stream: S,
    mut wsb: WSB,
    cb: impl FnOnce(&dyn crate::http::GenericRequest) -> bool,
  ) -> crate::Result<Self>
  where
    C: Compression<false, NegotiatedCompression = NC>,
  {
    let nb = &mut wsb.lease_mut().nb;
    nb._set_indices_through_expansion(0, 0, MAX_READ_LEN)?;
    let mut read = 0;
    loop {
      let read_buffer = nb._following_mut().get_mut(read..).unwrap_or_default();
      let local_read = stream.read(read_buffer).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedStreamReadEOF);
      }
      read = read.wrapping_add(local_read);
      let mut req_buffer = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
      let mut req = Request::new(&mut req_buffer);
      match req.parse(nb._following())? {
        Status::Complete(_) => {
          if !cb(&req) {
            return Err(WebSocketError::InvalidAcceptRequest.into());
          }
          if !_trim_bytes(req.method()).eq_ignore_ascii_case(b"get") {
            return Err(HttpError::UnexpectedHttpMethod { expected: Method::Get }.into());
          }
          verify_common_header(req.headers)?;
          if !has_header_key_and_value(req.headers, b"sec-websocket-version", b"13") {
            return Err(
              HttpError::MissingHeader { expected: KnownHeaderName::SecWebsocketVersion }.into(),
            );
          };
          let Some(key) = req.headers.iter().find_map(|el| {
            (el.name().eq_ignore_ascii_case(b"sec-websocket-key")).then_some(el.value())
          }) else {
            return Err(
              HttpError::MissingHeader { expected: KnownHeaderName::SecWebsocketKey }.into(),
            );
          };
          let compression = compression.negotiate(req.headers.iter())?;
          let mut key_buffer = [0; 30];
          let swa = derived_key(&mut key_buffer, key);
          let mut headers_buffer = HeadersBuffer::<_, 3>::default();
          headers_buffer._headers[0] = Header { name: "Connection", value: b"Upgrade" };
          headers_buffer._headers[1] = Header { name: "Sec-WebSocket-Accept", value: swa };
          headers_buffer._headers[2] = Header { name: "Upgrade", value: b"websocket" };
          let mut res = Response::new(&mut headers_buffer._headers);
          res.code = Some(101);
          res.version = Some(req.version().into());
          let mut fbw = nb.into();
          let res_bytes = build_res(&compression, &mut fbw, res.headers)?;
          stream.write_all(res_bytes).await?;
          nb._clear();
          return WebSocketServer::new(compression, rng, stream, wsb);
        }
        Status::Partial => {}
      }
    }
  }
}

impl<NC, RNG, S, WSB> WebSocketClient<NC, RNG, S, WSB>
where
  NC: NegotiatedCompression,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Sends data to establish an WebSocket connection.
  #[inline]
  pub async fn connect<'fb, 'hb, 'headers, C>(
    compression: C,
    fb: &'fb mut FrameBufferVec,
    headers: impl IntoIterator<Item = (&'headers [u8], &'headers [u8])>,
    headers_buffer: &'hb mut HeadersBuffer<Header<'fb>, MAX_READ_HEADER_LEN>,
    mut rng: RNG,
    mut stream: S,
    uri: &UriRef<'_>,
    mut wsb: WSB,
  ) -> crate::Result<(Response<'hb, 'fb>, WebSocketClient<C::NegotiatedCompression, RNG, S, WSB>)>
  where
    C: Compression<true, NegotiatedCompression = NC>,
  {
    let key_buffer = &mut [0; 26];
    let nb = &mut wsb.lease_mut().nb;
    nb._clear();
    let mut fbw = nb.into();
    let key = build_req(&compression, &mut fbw, headers, key_buffer, &mut rng, uri)?;
    stream.write_all(fbw._curr_bytes()).await?;
    let mut read = 0;
    fb._set_indices_through_expansion(0, 0, MAX_READ_LEN)?;
    let len = loop {
      let mut local_header = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
      let read_buffer = fb.payload_mut().get_mut(read..).unwrap_or_default();
      let local_read = stream.read(read_buffer).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedStreamReadEOF);
      }
      read = read.wrapping_add(local_read);
      match Response::new(&mut local_header).parse(fb.payload())? {
        Status::Complete(len) => break len,
        Status::Partial => {}
      }
    };
    let mut res = Response::new(&mut headers_buffer._headers);
    let _status = res.parse(fb.payload())?;
    if res.code != Some(101) {
      return Err(WebSocketError::MissingSwitchingProtocols.into());
    }
    verify_common_header(res.headers)?;
    if !has_header_key_and_value(
      res.headers,
      b"sec-websocket-accept",
      derived_key(&mut [0; 30], key),
    ) {
      return Err(HttpError::MissingHeader { expected: KnownHeaderName::SecWebsocketKey }.into());
    }
    let compression = compression.negotiate(res.headers.iter())?;
    nb._set_indices_through_expansion(0, 0, read.wrapping_sub(len))?;
    nb._following_mut().copy_from_slice(fb.payload().get(len..read).unwrap_or_default());
    Ok((res, WebSocketClient::new(compression, rng, stream, wsb)?))
  }
}

fn base64_from_array<'output, const I: usize, const O: usize>(
  input: &[u8; I],
  output: &'output mut [u8; O],
) -> &'output [u8] {
  const {
    let rslt = if let Some(elem) = base64::encoded_len(I, false) { elem } else { 0 };
    assert!(O >= rslt);
  }
  let len = STANDARD.encode_slice(input, output).unwrap_or_default();
  output.get(..len).unwrap_or_default()
}

/// Client request
fn build_req<'bytes, 'kb, C>(
  compression: &C,
  fbw: &mut FilledBufferWriter<'_>,
  headers: impl IntoIterator<Item = (&'bytes [u8], &'bytes [u8])>,
  key_buffer: &'kb mut [u8; 26],
  rng: &mut impl Rng,
  uri: &UriRef<'_>,
) -> Result<&'kb [u8], VectorError>
where
  C: Compression<true>,
{
  let key = gen_key(key_buffer, rng);
  fbw._extend_from_slices_group_rn(&[
    b"GET ",
    uri.relative_reference_slash().as_bytes(),
    b" HTTP/1.1",
  ])?;
  for (name, value) in headers {
    fbw._extend_from_slices_group_rn(&[name, b": ", value])?;
  }
  fbw._extend_from_slice_rn(b"Connection: Upgrade")?;
  match uri.port() {
    Some(80 | 443) => {
      fbw._extend_from_slices_group_rn(&[b"Host: ", uri.hostname().as_bytes()])?;
    }
    _ => fbw._extend_from_slices_group_rn(&[b"Host: ", uri.host().as_bytes()])?,
  }
  fbw._extend_from_slices_group_rn(&[b"Sec-WebSocket-Key: ", key])?;
  fbw._extend_from_slice_rn(b"Sec-WebSocket-Version: 13")?;
  fbw._extend_from_slice_rn(b"Upgrade: websocket")?;
  compression.write_req_headers(fbw)?;
  fbw._extend_from_slice_rn(b"")?;
  Ok(key)
}

/// Server response
fn build_res<'fpb, C>(
  compression: &C,
  fbw: &'fpb mut FilledBufferWriter<'fpb>,
  headers: &[Header<'_>],
) -> Result<&'fpb [u8], VectorError>
where
  C: NegotiatedCompression,
{
  fbw._extend_from_slice_rn(b"HTTP/1.1 101 Switching Protocols")?;
  for header in headers {
    fbw._extend_from_slices_group_rn(&[header.name(), b": ", header.value()])?;
  }
  compression.write_res_headers(fbw)?;
  fbw._extend_from_slice_rn(b"")?;
  Ok(fbw._curr_bytes())
}

fn derived_key<'buffer>(buffer: &'buffer mut [u8; 30], key: &[u8]) -> &'buffer [u8] {
  let mut sha1 = Sha1::new();
  sha1.update(key);
  sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
  base64_from_array(&sha1.finalize().into(), buffer)
}

fn gen_key<'buffer>(buffer: &'buffer mut [u8; 26], rng: &mut impl Rng) -> &'buffer [u8] {
  base64_from_array(&rng.u8_16(), buffer)
}

fn has_header_key_and_value(headers: &[Header<'_>], key: &[u8], value: &[u8]) -> bool {
  headers
    .iter()
    .find_map(|h| {
      let has_key = _trim_bytes(h.name()).eq_ignore_ascii_case(key);
      let has_value =
        bytes_split1(h.value(), b',').any(|el| _trim_bytes(el).eq_ignore_ascii_case(value));
      (has_key && has_value).then_some(true)
    })
    .unwrap_or(false)
}

fn verify_common_header(buffer: &[Header<'_>]) -> crate::Result<()> {
  if !has_header_key_and_value(buffer, b"connection", b"upgrade") {
    return Err(HttpError::MissingHeader { expected: KnownHeaderName::Connection }.into());
  }
  if !has_header_key_and_value(buffer, b"upgrade", b"websocket") {
    return Err(HttpError::MissingHeader { expected: KnownHeaderName::Upgrade }.into());
  }
  Ok(())
}
