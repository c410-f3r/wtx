#[cfg(all(feature = "_async-tests", test))]
mod tests;

macro_rules! check_headers {
  ($headers:expr, $($header:expr),*) => {{
    let rslt = check_headers(
      [
        (KnownHeaderName::Connection, Some(b"upgrade")),
        (KnownHeaderName::Upgrade, Some(b"websocket")),
        $($header,)*
      ],
      $headers
    )?;
    drop(check_header_value(rslt[0]));
    drop(check_header_value(rslt[1]));
    rslt
  }};
}

use crate::{
  http::{GenericHeader as _, GenericRequest as _, HttpError, KnownHeaderName, Method},
  misc::{LeaseMut, Rng, Stream, SuffixWriterFbvm, UriRef, bytes_split1},
  web_socket::{
    Compression, WebSocket, WebSocketAcceptor, WebSocketBuffer, WebSocketConnector, WebSocketError,
    compression::NegotiatedCompression,
  },
};
use base64::{Engine, engine::general_purpose::STANDARD};
use httparse::{EMPTY_HEADER, Header, Request, Response, Status};
use sha1::{Digest, Sha1};

const MAX_READ_HEADER_LEN: usize = 64;
const MAX_READ_LEN: usize = 2 * 1024;
const NO_MASKING: &str = "no-masking";
const UPGRADE: &str = "Upgrade";
const VERSION: &str = "13";
const WEBSOCKET: &str = "websocket";

impl<C, E, R, WSB> WebSocketAcceptor<C, R, WSB>
where
  C: Compression<false>,
  E: From<crate::Error>,
  R: FnOnce(&Request<'_, '_>) -> Result<(), E>,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Reads external data to establish an WebSocket connection.
  #[inline]
  pub async fn accept<S>(
    mut self,
    mut stream: S,
  ) -> Result<WebSocket<C::NegotiatedCompression, S, WSB, false>, E>
  where
    S: Stream,
  {
    self.wsb.lease_mut()._clear();
    let nb = &mut self.wsb.lease_mut().network_buffer;
    nb._reserve(MAX_READ_LEN)?;
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
          (self.req)(&req)?;
          if !req.method().trim_ascii().eq_ignore_ascii_case(b"get") {
            return Err(
              crate::Error::from(HttpError::UnexpectedHttpMethod { expected: Method::Get }).into(),
            );
          }
          let mut key_buffer = [0; 30];
          let [_, _, c, d, e] = check_headers!(
            req.headers,
            (KnownHeaderName::SecWebsocketExtensions, None),
            (KnownHeaderName::SecWebsocketKey, None),
            (KnownHeaderName::SecWebsocketVersion, Some(VERSION.as_bytes()))
          );
          self.no_masking &= check_header_value(c).is_ok_and(has_no_masking);
          let key = check_header_value(d)?;
          let _ = check_header_value(e)?;
          let nc = self.compression.negotiate(req.headers.iter())?;
          let swa = derived_key(&mut key_buffer, key);
          let mut headers_buffer = [EMPTY_HEADER; 3];
          headers_buffer[0] = Header { name: "Connection", value: UPGRADE.as_bytes() };
          headers_buffer[1] = Header { name: "Sec-WebSocket-Accept", value: swa };
          headers_buffer[2] = Header { name: "Upgrade", value: WEBSOCKET.as_bytes() };
          let mut res = Response::new(&mut headers_buffer);
          res.code = Some(101);
          res.version = Some(req.version().into());
          {
            let mut sw = nb._suffix_writer();
            build_res(&mut sw, res.headers, &nc, self.no_masking)?;
            stream.write_all(sw._curr_bytes()).await?;
          }
          nb._clear();
          return Ok(WebSocket::new(nc, self.no_masking, self.rng, stream, self.wsb)?);
        }
        Status::Partial => {}
      }
    }
  }
}

impl<'headers, C, E, H, R, WSB> WebSocketConnector<C, H, R, WSB>
where
  C: Compression<true>,
  E: From<crate::Error>,
  H: IntoIterator<Item = (&'headers [u8], &'headers [u8])>,
  R: FnOnce(&Response<'_, '_>) -> Result<(), E>,
  WSB: LeaseMut<WebSocketBuffer>,
{
  /// Sends data to establish an WebSocket connection.
  #[inline]
  pub async fn connect<S>(
    mut self,
    mut stream: S,
    uri: &UriRef<'_>,
  ) -> Result<WebSocket<C::NegotiatedCompression, S, WSB, true>, E>
  where
    S: Stream,
  {
    self.wsb.lease_mut()._clear();
    let key_buffer = &mut [0; 26];
    let key = {
      let nb = &mut self.wsb.lease_mut().network_buffer;
      nb._reserve(MAX_READ_LEN)?;
      {
        let mut sw = nb._suffix_writer();
        let key = build_req(
          &self.compression,
          &mut sw,
          self.headers,
          key_buffer,
          self.no_masking,
          &mut self.rng,
          uri,
        )?;
        stream.write_all(sw._curr_bytes()).await?;
        key
      }
    };
    let mut read = 0;
    let (nc, len) = loop {
      let nb = &mut self.wsb.lease_mut().network_buffer;
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
      (self.res)(&res)?;
      let [_, _, c, d] = check_headers!(
        res.headers,
        (KnownHeaderName::SecWebsocketAccept, Some(derived_key(&mut [0; 30], key))),
        (KnownHeaderName::SecWebsocketExtensions, None)
      );
      drop(check_header_value(c));
      self.no_masking &= check_header_value(d).is_ok_and(has_no_masking);
      break (self.compression.negotiate(res.headers.iter())?, len);
    };
    self.wsb.lease_mut().network_buffer._set_indices(0, len, read.wrapping_sub(len))?;
    Ok(WebSocket::new(nc, self.no_masking, self.rng, stream, self.wsb)?)
  }
}

#[inline]
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
#[inline]
fn build_req<'headers, 'kb, C>(
  compression: &C,
  sw: &mut SuffixWriterFbvm<'_>,
  headers: impl IntoIterator<Item = (&'headers [u8], &'headers [u8])>,
  key_buffer: &'kb mut [u8; 26],
  no_masking: bool,
  rng: &mut impl Rng,
  uri: &UriRef<'_>,
) -> crate::Result<&'kb [u8]>
where
  C: Compression<true>,
{
  let key = gen_key(key_buffer, rng);
  sw._extend_from_slices_group_rn(&[
    b"GET ",
    uri.relative_reference_slash().as_bytes(),
    b" HTTP/1.1",
  ])?;
  for (name, value) in headers {
    sw._extend_from_slices_group_rn(&[name, b": ", value])?;
  }
  sw._extend_from_slice_rn(b"Connection: Upgrade")?;
  match uri.port() {
    Some(80 | 443) => {
      sw._extend_from_slices_group_rn(&[b"Host: ", uri.hostname().as_bytes()])?;
    }
    _ => sw._extend_from_slices_group_rn(&[b"Host: ", uri.host().as_bytes()])?,
  }
  sw._extend_from_slices_group_rn(&[b"Sec-WebSocket-Key: ", key])?;
  if no_masking {
    sw._extend_from_slice_rn(b"Sec-WebSocket-Extensions: no-masking")?;
  }
  sw._extend_from_slice_rn(b"Sec-WebSocket-Version: 13")?;
  sw._extend_from_slice_rn(b"Upgrade: websocket")?;
  compression.write_req_headers(sw)?;
  sw._extend_from_slice_rn(b"")?;
  Ok(key)
}

/// Server response
#[inline]
fn build_res<NC>(
  sw: &mut SuffixWriterFbvm<'_>,
  headers: &[Header<'_>],
  nc: &NC,
  no_masking: bool,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
{
  sw._extend_from_slice_rn(b"HTTP/1.1 101 Switching Protocols")?;
  for header in headers {
    sw._extend_from_slices_group_rn(&[header.name(), b": ", header.value()])?;
  }
  if no_masking {
    sw._extend_from_slices_group_rn(&[
      KnownHeaderName::SecWebsocketExtensions.into(),
      b": ",
      NO_MASKING.as_bytes(),
    ])?;
  }
  nc.write_res_headers(sw)?;
  sw._extend_from_slice_rn(b"")?;
  Ok(())
}

#[inline]
fn check_header_value((name, value): (KnownHeaderName, Option<&[u8]>)) -> crate::Result<&[u8]> {
  let Some(elem) = value else {
    return Err(crate::Error::from(HttpError::MissingHeader(name)));
  };
  Ok(elem)
}

#[inline]
fn check_headers<'headers, const N: usize>(
  array: [(KnownHeaderName, Option<&[u8]>); N],
  headers: &'headers [Header<'_>],
) -> crate::Result<[(KnownHeaderName, Option<&'headers [u8]>); N]> {
  let mut rslt = [(KnownHeaderName::Accept, None); N];
  for header in headers {
    let trimmed_name = header.name().trim_ascii();
    let trimmed_value = header.value().trim_ascii();
    for ((name, value_opt), rslt_elem) in array.into_iter().zip(&mut rslt) {
      let has_name = rslt_elem.1.is_none() && trimmed_name.eq_ignore_ascii_case(name.into());
      if has_name {
        if let Some(value) = value_opt {
          for sub_value in bytes_split1(trimmed_value, b',') {
            if sub_value.trim_ascii().eq_ignore_ascii_case(value) {
              *rslt_elem = (name, Some(sub_value));
              break;
            }
          }
          if rslt_elem.1.is_some() {
            break;
          }
        } else {
          *rslt_elem = (name, Some(trimmed_value));
        }
      }
    }
  }
  Ok(rslt)
}

#[inline]
fn derived_key<'buffer>(buffer: &'buffer mut [u8; 30], key: &[u8]) -> &'buffer [u8] {
  let mut sha1 = Sha1::new();
  sha1.update(key);
  sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
  base64_from_array(&sha1.finalize().into(), buffer)
}

#[inline]
fn gen_key<'buffer>(buffer: &'buffer mut [u8; 26], rng: &mut impl Rng) -> &'buffer [u8] {
  base64_from_array(&rng.u8_16(), buffer)
}

#[inline]
fn has_no_masking(el: &[u8]) -> bool {
  el.eq_ignore_ascii_case(NO_MASKING.as_bytes())
}
