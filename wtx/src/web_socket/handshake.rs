#[cfg(all(feature = "_async-tests", test))]
mod tests;

use crate::{
  http::{GenericHeader as _, GenericRequest as _, HttpError, KnownHeaderName, Method},
  misc::{bytes_split1, FilledBufferWriter, LeaseMut, Rng, Stream, UriRef, VectorError},
  web_socket::{
    compression::NegotiatedCompression, misc::_trim_bytes, Compression, WebSocketBuffer,
    WebSocketClient, WebSocketError, WebSocketServer,
  },
};
use base64::{engine::general_purpose::STANDARD, Engine};
use httparse::{Header, Request, Response, Status, EMPTY_HEADER};
use sha1::{Digest, Sha1};

const MAX_READ_LEN: usize = 2 * 1024;
const MAX_READ_HEADER_LEN: usize = 64;

impl<NC, RNG, S, WSB> WebSocketServer<NC, RNG, S, WSB>
where
  NC: NegotiatedCompression,
  RNG: Rng,
  S: Stream,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Reads external data to establish an WebSocket connection.
  #[inline]
  pub async fn accept<C, E>(
    compression: C,
    rng: RNG,
    mut stream: S,
    mut wsb: WSB,
    req_cb: impl FnOnce(&Request<'_, '_>) -> Result<(), E>,
  ) -> Result<Self, E>
  where
    C: Compression<false, NegotiatedCompression = NC>,
    E: From<crate::Error>,
  {
    wsb.lease_mut()._clear();
    let nb = &mut wsb.lease_mut().nb;
    nb._expand_buffer(MAX_READ_LEN).map_err(From::from)?;
    let mut read = 0;
    loop {
      let read_buffer = nb._buffer_mut().get_mut(read..).unwrap_or_default();
      let local_read = stream.read(read_buffer).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedStreamReadEOF.into());
      }
      read = read.wrapping_add(local_read);
      let mut req_buffer = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
      let mut req = Request::new(&mut req_buffer);
      match req.parse(nb._following()).map_err(From::from)? {
        Status::Complete(_) => {
          req_cb(&req)?;
          if !_trim_bytes(req.method()).eq_ignore_ascii_case(b"get") {
            return Err(
              crate::Error::from(HttpError::UnexpectedHttpMethod { expected: Method::Get }).into(),
            );
          }
          verify_common_header(req.headers)?;
          if !has_header_key_and_value(req.headers, b"sec-websocket-version", b"13") {
            let expected = KnownHeaderName::SecWebsocketVersion;
            return Err(crate::Error::from(HttpError::MissingHeader(expected)).into());
          };
          let Some(key) = req.headers.iter().find_map(|el| {
            (el.name().eq_ignore_ascii_case(b"sec-websocket-key")).then_some(el.value())
          }) else {
            return Err(
              crate::Error::from(HttpError::MissingHeader(KnownHeaderName::SecWebsocketKey)).into(),
            );
          };
          let compression = compression.negotiate(req.headers.iter())?;
          let mut key_buffer = [0; 30];
          let swa = derived_key(&mut key_buffer, key);
          let mut headers_buffer = [EMPTY_HEADER; 3];
          headers_buffer[0] = Header { name: "Connection", value: b"Upgrade" };
          headers_buffer[1] = Header { name: "Sec-WebSocket-Accept", value: swa };
          headers_buffer[2] = Header { name: "Upgrade", value: b"websocket" };
          let mut res = Response::new(&mut headers_buffer);
          res.code = Some(101);
          res.version = Some(req.version().into());
          {
            let mut fbw = nb.into();
            build_res(&compression, &mut fbw, res.headers).map_err(From::from)?;
            stream.write_all(fbw._curr_bytes()).await?;
          }
          nb._clear();
          return WebSocketServer::new(compression, rng, stream, wsb).map_err(From::from);
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
  pub async fn connect<'headers, C, E>(
    compression: C,
    headers: impl IntoIterator<Item = (&'headers [u8], &'headers [u8])>,
    mut rng: RNG,
    mut stream: S,
    uri: &UriRef<'_>,
    mut wsb: WSB,
    res_cb: impl FnOnce(&Response<'_, '_>) -> Result<(), E>,
  ) -> Result<WebSocketClient<C::NegotiatedCompression, RNG, S, WSB>, E>
  where
    C: Compression<true, NegotiatedCompression = NC>,
    E: From<crate::Error>,
  {
    wsb.lease_mut()._clear();
    let key_buffer = &mut [0; 26];
    let key = {
      let nb = &mut wsb.lease_mut().nb;
      nb._expand_buffer(MAX_READ_LEN).map_err(From::from)?;
      {
        let fbw = &mut nb.into();
        let key =
          build_req(&compression, fbw, headers, key_buffer, &mut rng, uri).map_err(From::from)?;
        stream.write_all(fbw._curr_bytes()).await?;
        key
      }
    };
    let mut read = 0;
    let (compression, len) = loop {
      let nb = &mut wsb.lease_mut().nb;
      let local_read = stream.read(nb._buffer_mut().get_mut(read..).unwrap_or_default()).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedStreamReadEOF.into());
      }
      read = read.wrapping_add(local_read);
      let mut httparse_headers = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
      let mut res = Response::new(&mut httparse_headers);
      let len = match res.parse(nb._buffer().get(..read).unwrap_or_default()).map_err(From::from)? {
        Status::Complete(len) => len,
        Status::Partial => continue,
      };
      if res.code != Some(101) {
        return Err(crate::Error::from(WebSocketError::MissingSwitchingProtocols).into());
      }
      res_cb(&res)?;
      verify_common_header(res.headers)?;
      if !has_header_key_and_value(
        res.headers,
        b"sec-websocket-accept",
        derived_key(&mut [0; 30], key),
      ) {
        return Err(
          crate::Error::from(HttpError::MissingHeader(KnownHeaderName::SecWebsocketKey)).into(),
        );
      }
      break (compression.negotiate(res.headers.iter())?, len);
    };
    wsb.lease_mut().nb._set_indices(0, len, read.wrapping_sub(len))?;
    Ok(WebSocketClient::new(compression, rng, stream, wsb)?)
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
fn build_req<'headers, 'kb, C>(
  compression: &C,
  fbw: &mut FilledBufferWriter<'_>,
  headers: impl IntoIterator<Item = (&'headers [u8], &'headers [u8])>,
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
fn build_res<C>(
  compression: &C,
  fbw: &mut FilledBufferWriter<'_>,
  headers: &[Header<'_>],
) -> Result<(), VectorError>
where
  C: NegotiatedCompression,
{
  fbw._extend_from_slice_rn(b"HTTP/1.1 101 Switching Protocols")?;
  for header in headers {
    fbw._extend_from_slices_group_rn(&[header.name(), b": ", header.value()])?;
  }
  compression.write_res_headers(fbw)?;
  fbw._extend_from_slice_rn(b"")?;
  Ok(())
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
    return Err(HttpError::MissingHeader(KnownHeaderName::Connection).into());
  }
  if !has_header_key_and_value(buffer, b"upgrade", b"websocket") {
    return Err(HttpError::MissingHeader(KnownHeaderName::Upgrade).into());
  }
  Ok(())
}
