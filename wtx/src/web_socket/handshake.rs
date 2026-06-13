#[cfg(test)]
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
  codec::{Base64Alphabet, base64_encode, base64_encoded_len},
  collection::Vector,
  crypto::{Hash, Sha1HashGlobal},
  http::{GenericHeader as _, GenericRequest as _, HttpError, KnownHeaderName, Method},
  misc::{LeaseMut, PartitionedFilledBuffer, SuffixWriterFbvm, UriRef, bytes_split1},
  rng::{CryptoRng, Rng, SeedableRng as _, Xorshift64},
  stream::{Stream, StreamReader, StreamWriter},
  tls::{TlsAcceptor, TlsConfig, TlsConnector, TlsMode},
  web_socket::{
    Compression, WebSocket, WebSocketAcceptor, WebSocketBuffer, WebSocketConnector, WebSocketError,
    compression::NegotiatedCompression,
  },
};
use httparse::{EMPTY_HEADER, Header, Request, Response, Status};

const MAX_READ_HEADER_LEN: usize = 64;
const MAX_READ_LEN: usize = 2 * 1024;
const NO_MASKING: &str = "no-masking";
const UPGRADE: &str = "Upgrade";
const VERSION: &str = "13";
const WEBSOCKET: &str = "websocket";

impl<C, E, R, WB> WebSocketAcceptor<C, R, WB>
where
  C: Compression<false>,
  E: From<crate::Error>,
  R: FnOnce(&Request<'_, '_>) -> Result<bool, E>,
  WB: LeaseMut<WebSocketBuffer>,
{
  /// Reads external data to establish an WebSocket connection.
  #[inline]
  pub async fn accept<RNG, S, TM>(
    mut self,
    rng: &mut RNG,
    tls_acceptor: TlsAcceptor<S, TM>,
    tls_config: &TlsConfig<'_>,
  ) -> Result<WebSocket<C::NegotiatedCompression, S, TM, WB, false>, E>
  where
    RNG: CryptoRng,
    S: Stream,
    TM: TlsMode,
  {
    self.wsb.lease_mut().clear();
    let mut tls_stream = tls_acceptor
      .accept(&mut self.wsb.lease_mut().network_buffer, rng, tls_config, &mut Vector::new())
      .await?;
    let nb = &mut self.wsb.lease_mut().network_buffer;
    nb.reserve(MAX_READ_LEN)?;
    let mut read = 0;
    loop {
      let read_buffer = nb.all_mut().get_mut(read..).unwrap_or_default();
      let local_read = tls_stream.read(read_buffer).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedStreamReadEOF.into());
      }
      read = read.wrapping_add(local_read);
      let mut req_buffer = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
      let mut req = Request::new(&mut req_buffer);
      match req.parse(nb.all().get(..read).unwrap_or_default()).map_err(From::from)? {
        Status::Complete(len) => {
          if !(self.req)(&req)? {
            build_and_send_res400(nb, &mut tls_stream).await?;
            return Err(crate::Error::from(WebSocketError::ClosedHandshake).into());
          }
          if !req.method().trim_ascii().eq_ignore_ascii_case(b"get") {
            build_and_send_res400(nb, &mut tls_stream).await?;
            return Err(
              crate::Error::from(HttpError::UnexpectedHttpMethod { expected: Method::Get }).into(),
            );
          }
          let mut key_buffer = [0; 30];
          let swa = match Self::check_req_headers(&mut self.no_masking, &req, &mut key_buffer) {
            Ok(el) => el,
            Err(err) => {
              build_and_send_res400(nb, &mut tls_stream).await?;
              return Err(err.into());
            }
          };
          let nc = self.compression.negotiate(req.headers.iter())?;
          let mut headers_buffer = [EMPTY_HEADER; 3];
          headers_buffer[0] = Header { name: "Connection", value: UPGRADE.as_bytes() };
          headers_buffer[1] = Header { name: "Sec-WebSocket-Accept", value: swa };
          headers_buffer[2] = Header { name: "Upgrade", value: WEBSOCKET.as_bytes() };
          let mut res = Response::new(&mut headers_buffer);
          res.code = Some(101);
          res.version = Some(req.version().into());
          {
            let mut sw = nb.suffix_writer();
            build_res101(&mut sw, res.headers, &nc, self.no_masking)?;
            tls_stream.write_all(sw.curr_bytes()).await?;
          }
          self.wsb.lease_mut().network_buffer.set_indices(0, len, read.wrapping_sub(len))?;
          return Ok(WebSocket::new(
            nc,
            self.no_masking,
            Xorshift64::from_simple_seed()?,
            tls_stream,
            self.wsb,
          ));
        }
        Status::Partial => {}
      }
    }
  }

  fn check_req_headers<'kb>(
    no_masking: &mut bool,
    req: &Request<'_, '_>,
    key_buffer: &'kb mut [u8; 30],
  ) -> crate::Result<&'kb [u8]> {
    let [_, _, c, d, e] = check_headers!(
      req.headers,
      (KnownHeaderName::SecWebsocketExtensions, None),
      (KnownHeaderName::SecWebsocketKey, None),
      (KnownHeaderName::SecWebsocketVersion, Some(VERSION.as_bytes()))
    );
    *no_masking &= check_header_value(c).is_ok_and(has_no_masking);
    let key = check_header_value(d)?;
    let _ = check_header_value(e)?;
    Ok(derived_key(key_buffer, key))
  }
}

impl<'headers, C, E, H, R, WB> WebSocketConnector<C, H, R, WB>
where
  C: Compression<true>,
  E: From<crate::Error>,
  H: IntoIterator<Item = (&'headers str, &'headers str)>,
  R: FnOnce(&Response<'_, '_>) -> Result<(), E>,
  WB: LeaseMut<WebSocketBuffer>,
{
  /// Sends data to establish an WebSocket connection.
  #[inline]
  pub async fn connect<RNG, S, TM>(
    mut self,
    rng: &mut RNG,
    tls_connector: TlsConnector<S, TM>,
    tls_config: &TlsConfig<'_>,
    uri: &UriRef<'_>,
  ) -> Result<WebSocket<C::NegotiatedCompression, S, TM, WB, true>, E>
  where
    RNG: CryptoRng,
    S: Stream,
    TM: TlsMode,
  {
    self.wsb.lease_mut().clear();
    let mut tls_stream = tls_connector
      .connect(&mut self.wsb.lease_mut().network_buffer, None, rng, tls_config, &mut Vector::new())
      .await?;
    let key_buffer = &mut [0; 26];
    let mut rng = Xorshift64::from_simple_seed()?;
    let key = {
      let nb = &mut self.wsb.lease_mut().network_buffer;
      nb.reserve(MAX_READ_LEN)?;
      {
        let mut sw = nb.suffix_writer();
        let key = build_req(
          &self.compression,
          &mut sw,
          self.headers,
          key_buffer,
          self.no_masking,
          &mut rng,
          uri,
        )?;
        tls_stream.write_all(sw.curr_bytes()).await?;
        key
      }
    };
    let mut read = 0;
    let (nc, len) = loop {
      let nb = &mut self.wsb.lease_mut().network_buffer;
      let local_read = tls_stream.read(nb.all_mut().get_mut(read..).unwrap_or_default()).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedStreamReadEOF.into());
      }
      read = read.wrapping_add(local_read);
      let mut httparse_headers = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
      let mut res = Response::new(&mut httparse_headers);
      let len = match res.parse(nb.all().get(..read).unwrap_or_default()).map_err(From::from)? {
        Status::Complete(len) => len,
        Status::Partial => continue,
      };
      if res.code != Some(101) {
        return Err(
          crate::Error::from(WebSocketError::MissingSwitchingProtocols { found: res.code }).into(),
        );
      }
      (self.res_cb)(&res)?;
      let [_, _, c, d] = check_headers!(
        res.headers,
        (KnownHeaderName::SecWebsocketAccept, Some(derived_key(&mut [0; 30], key))),
        (KnownHeaderName::SecWebsocketExtensions, None)
      );
      drop(check_header_value(c));
      self.no_masking &= check_header_value(d).is_ok_and(has_no_masking);
      break (self.compression.negotiate(res.headers.iter())?, len);
    };
    self.wsb.lease_mut().network_buffer.set_indices(0, len, read.wrapping_sub(len))?;
    Ok(WebSocket::new(nc, self.no_masking, rng, tls_stream, self.wsb))
  }
}

fn base64_from_array<'output, const I: usize, const O: usize>(
  input: &[u8; I],
  output: &'output mut [u8; O],
) -> &'output [u8] {
  const {
    let rslt = if let Some(elem) = base64_encoded_len(I, false) { elem } else { 0 };
    assert!(O >= rslt);
  }
  base64_encode(Base64Alphabet::Standard, input, output).map(|el| el.as_bytes()).unwrap_or_default()
}

/// Client request
fn build_req<'headers, 'kb, C, RNG>(
  compression: &C,
  sw: &mut SuffixWriterFbvm<'_>,
  headers: impl IntoIterator<Item = (&'headers str, &'headers str)>,
  key_buffer: &'kb mut [u8; 26],
  no_masking: bool,
  rng: &mut RNG,
  uri: &UriRef<'_>,
) -> crate::Result<&'kb [u8]>
where
  C: Compression<true>,
  RNG: Rng,
{
  let key = gen_key(key_buffer, rng);
  sw.extend_from_slices_group_rn(&[
    b"GET ",
    uri.relative_reference_slash().as_bytes(),
    b" HTTP/1.1",
  ])?;
  sw.extend_from_slice_rn(b"Connection: Upgrade")?;
  match uri.port() {
    Some(80 | 443) => {
      sw.extend_from_slices_group_rn(&[b"Host: ", uri.hostname().as_bytes()])?;
    }
    _ => sw.extend_from_slices_group_rn(&[b"Host: ", uri.host().as_bytes()])?,
  }
  sw.extend_from_slices_group_rn(&[b"Sec-WebSocket-Key: ", key])?;
  if no_masking {
    sw.extend_from_slice_rn(b"Sec-WebSocket-Extensions: no-masking")?;
  }
  sw.extend_from_slice_rn(b"Sec-WebSocket-Version: 13")?;
  sw.extend_from_slice_rn(b"Upgrade: websocket")?;
  for (name, value) in headers {
    sw.extend_from_slices_group_rn(&[name.as_bytes(), b": ", value.as_bytes()])?;
  }
  compression.write_req_headers(sw)?;
  sw.extend_from_slice_rn(b"")?;
  Ok(key)
}

async fn build_and_send_res400<S>(
  nb: &mut PartitionedFilledBuffer,
  stream: &mut S,
) -> crate::Result<()>
where
  S: Stream,
{
  let mut sw = nb.suffix_writer();
  sw.extend_from_slice_rn(b"HTTP/1.1 400 Bad Request")?;
  sw.extend_from_slice_rn(b"")?;
  stream.write_all(sw.curr_bytes()).await?;
  Ok(())
}

/// Server response
fn build_res101<NC>(
  sw: &mut SuffixWriterFbvm<'_>,
  headers: &[Header<'_>],
  nc: &NC,
  no_masking: bool,
) -> crate::Result<()>
where
  NC: NegotiatedCompression,
{
  sw.extend_from_slice_rn(b"HTTP/1.1 101 Switching Protocols")?;
  for header in headers {
    sw.extend_from_slices_group_rn(&[header.name(), b": ", header.value()])?;
  }
  if no_masking {
    sw.extend_from_slices_group_rn(&[
      KnownHeaderName::SecWebsocketExtensions.into(),
      b": ",
      NO_MASKING.as_bytes(),
    ])?;
  }
  nc.write_res_headers(sw)?;
  sw.extend_from_slice_rn(b"")?;
  Ok(())
}

fn check_header_value((name, value): (KnownHeaderName, Option<&[u8]>)) -> crate::Result<&[u8]> {
  let Some(elem) = value else {
    return Err(crate::Error::from(HttpError::MissingHeader(name)));
  };
  Ok(elem)
}

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

fn derived_key<'buffer>(buffer: &'buffer mut [u8; 30], key: &[u8]) -> &'buffer [u8] {
  let array = Sha1HashGlobal::digest([key, b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"]);
  base64_from_array(&array, buffer)
}

fn gen_key<'buffer>(buffer: &'buffer mut [u8; 26], rng: &mut impl Rng) -> &'buffer [u8] {
  base64_from_array(&rng.u8_16(), buffer)
}

const fn has_no_masking(el: &[u8]) -> bool {
  el.eq_ignore_ascii_case(NO_MASKING.as_bytes())
}
