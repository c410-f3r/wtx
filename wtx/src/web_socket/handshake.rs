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
    );
    drop(check_header_value(rslt[0]));
    drop(check_header_value(rslt[1]));
    rslt
  }};
}

use crate::{
  codec::{Base64Alphabet, base64_encode, base64_encoded_len},
  collections::SuffixPusherVectorMut,
  crypto::{Hash as _, Sha1HashGlobal},
  http::{GenericHeader as _, GenericRequest as _, HttpError, KnownHeaderName, Method},
  misc::{Lease, SingleTypeStorage, UriRef, bytes_split1},
  rng::{CryptoRng, Rng, SeedableRng as _, Xorshift64},
  stream::{Stream, StreamWriter as _},
  tls::{TlsAcceptor, TlsConfig, TlsConnector, TlsMode},
  web_socket::{
    WebSocket, WebSocketAcceptor, WebSocketConnector, WebSocketError, WsCompression,
    web_socket_compression::NegotiatedWsCompression,
  },
};
use httparse::{EMPTY_HEADER, Header, Request, Response, Status};

const MAX_HEADERS: usize = 16;
const READ_INCREMENT: usize = 1024;
const VERSION: &str = "13";

impl<C, E, R> WebSocketAcceptor<C, R>
where
  C: WsCompression<false>,
  E: From<crate::Error>,
  R: FnOnce(&Request<'_, '_>) -> Result<bool, E>,
{
  /// Reads external data to establish an WebSocket connection.
  #[inline]
  pub async fn accept<RNG, S, TC, TM>(
    mut self,
    tls_acceptor: TlsAcceptor<RNG, S, TC>,
  ) -> Result<WebSocket<C::NegotiatedCompression, S, TM, false>, E>
  where
    RNG: CryptoRng,
    S: Stream,
    TC: Lease<TlsConfig<TM>> + SingleTypeStorage<Item = TM>,
    TM: TlsMode,
  {
    self.wsb.clear();
    let mut tls_stream = tls_acceptor.accept().await?.rslt()?.stream;
    let nb = &mut self.wsb.network_buffer;
    loop {
      let _ = nb.read_arbitrary(READ_INCREMENT, &mut tls_stream).await?.rslt()?;
      let mut req_buffer = [EMPTY_HEADER; MAX_HEADERS];
      let mut req = Request::new(&mut req_buffer);
      let buffer = nb.current();
      match req.parse(buffer).map_err(From::from)? {
        Status::Complete(len) => {
          if !(self.req)(&req)? {
            build_and_send_res400(&mut tls_stream).await?;
            return Err(crate::Error::from(WebSocketError::ClosedHandshake).into());
          }
          if !req.method().trim_ascii().eq_ignore_ascii_case(b"get") {
            build_and_send_res400(&mut tls_stream).await?;
            return Err(
              crate::Error::from(HttpError::UnexpectedHttpMethod { expected: Method::Get }).into(),
            );
          }
          let mut key_buffer = [0; 30];
          let swa = match Self::check_req_headers(&mut self.no_masking, &req, &mut key_buffer) {
            Ok(el) => el,
            Err(err) => {
              build_and_send_res400(&mut tls_stream).await?;
              return Err(err.into());
            }
          };
          let nc = self.compression.negotiate(req.headers.iter())?;
          build_and_send_res101(swa, &nc, self.no_masking, &mut tls_stream).await?;
          self.wsb.network_buffer.set_indices(len, len);
          let rng = Xorshift64::from_simple_seed()?;
          return Ok(WebSocket::new(nc, self.no_masking, rng, tls_stream, self.wsb));
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
    let [_, _, b2, b3, b4] = check_headers!(
      req.headers,
      (KnownHeaderName::SecWebsocketExtensions, None),
      (KnownHeaderName::SecWebsocketKey, None),
      (KnownHeaderName::SecWebsocketVersion, Some(VERSION.as_bytes()))
    );
    *no_masking &= check_header_value(b2).is_ok_and(has_no_masking);
    let key = check_header_value(b3)?;
    let _ = check_header_value(b4)?;
    Ok(derived_key(key_buffer, key))
  }
}

impl<'headers, C, E, H, R> WebSocketConnector<C, H, R>
where
  C: WsCompression<true>,
  E: From<crate::Error>,
  H: IntoIterator<Item = (&'headers str, &'headers str)>,
  R: FnOnce(&Response<'_, '_>) -> Result<(), E>,
{
  /// Sends data to establish an WebSocket connection.
  #[inline]
  pub async fn connect<RNG, S, TC, TM>(
    mut self,
    tls_connector: TlsConnector<RNG, S, TC>,
    uri: &UriRef<'_>,
  ) -> Result<WebSocket<C::NegotiatedCompression, S, TM, true>, E>
  where
    RNG: CryptoRng,
    S: Stream,
    TC: Lease<TlsConfig<TM>> + SingleTypeStorage<Item = TM>,
    TM: TlsMode,
  {
    self.wsb.clear();
    let mut tls_stream = tls_connector.connect().await?.rslt()?;
    let key_buffer = &mut [0; 26];
    let key = {
      let mut sw = self.wsb.network_buffer.suffix_pusher();
      let key = build_req(
        &self.compression,
        &mut sw,
        self.headers,
        key_buffer,
        self.no_masking,
        &mut tls_stream.rng,
        uri,
      )?;
      tls_stream.stream.write_all(sw.curr()).await?;
      key
    };
    let (nc, len) = loop {
      let nb = &mut self.wsb.network_buffer;
      let _ = nb.read_arbitrary(READ_INCREMENT, &mut tls_stream.stream).await?.rslt()?;
      let mut httparse_headers = [EMPTY_HEADER; MAX_HEADERS];
      let mut res = Response::new(&mut httparse_headers);
      let buffer = nb.current();
      let len = match res.parse(buffer).map_err(From::from)? {
        Status::Complete(len) => len,
        Status::Partial => continue,
      };
      if res.code != Some(101) {
        return Err(
          crate::Error::from(WebSocketError::MissingSwitchingProtocols { found: res.code }).into(),
        );
      }
      (self.res_cb)(&res)?;
      let [_, _, b2, b3] = check_headers!(
        res.headers,
        (KnownHeaderName::SecWebsocketAccept, Some(derived_key(&mut [0; 30], key))),
        (KnownHeaderName::SecWebsocketExtensions, None)
      );
      drop(check_header_value(b2));
      self.no_masking &= check_header_value(b3).is_ok_and(has_no_masking);
      break (self.compression.negotiate(res.headers.iter())?, len);
    };
    self.wsb.network_buffer.set_indices(len, len);
    let rng = Xorshift64::from_simple_seed()?;
    Ok(WebSocket::new(nc, self.no_masking, rng, tls_stream.stream, self.wsb))
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
  base64_encode(Base64Alphabet::Standard, input, output).map(str::as_bytes).unwrap_or_default()
}

/// Client request
fn build_req<'headers, 'kb, C, RNG>(
  compression: &C,
  sw: &mut SuffixPusherVectorMut<'_, u8>,
  headers: impl IntoIterator<Item = (&'headers str, &'headers str)>,
  key_buffer: &'kb mut [u8; 26],
  no_masking: bool,
  rng: &mut RNG,
  uri: &UriRef<'_>,
) -> crate::Result<&'kb [u8]>
where
  C: WsCompression<true>,
  RNG: CryptoRng,
{
  let host = match uri.port() {
    Some(80 | 443) => uri.hostname().as_bytes(),
    _ => uri.host().as_bytes(),
  };
  let key = gen_key(key_buffer, rng);
  let _ = sw.inner_mut().extend_from_copyable_slices(&[
    b"GET ",
    uri.relative_reference_slash().as_bytes(),
    b" HTTP/1.1\r\n",
    b"Connection: Upgrade\r\n",
    b"Host: ",
    host,
    b"\r\n",
    b"Sec-WebSocket-Key: ",
    key,
    b"\r\n",
    no_masking_bytes(no_masking),
    b"Sec-WebSocket-Version: 13\r\n",
    b"Upgrade: websocket\r\n",
    compression.req_headers().as_ref(),
  ])?;
  for (name, value) in headers {
    let _ = sw.inner_mut().extend_from_copyable_slices(&[
      name.as_bytes(),
      b": ",
      value.as_bytes(),
      b"\r\n",
    ])?;
  }
  sw.inner_mut().extend_from_copyable_slice(b"\r\n")?;
  Ok(key)
}

async fn build_and_send_res400<S>(stream: &mut S) -> crate::Result<()>
where
  S: Stream,
{
  stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await?;
  Ok(())
}

/// Server response
async fn build_and_send_res101<NC, S>(
  key: &[u8],
  nc: &NC,
  no_masking: bool,
  stream: &mut S,
) -> crate::Result<()>
where
  NC: NegotiatedWsCompression,
  S: Stream,
{
  let begin = b"HTTP/1.1 101 Switching Protocols\r\n";
  let middle = b"Connection: Upgrade\r\nSec-WebSocket-Accept: ";
  let end = b"\r\nUpgrade: websocket\r\n\r\n";
  stream
    .write_all_vectored(&[
      begin,
      no_masking_bytes(no_masking),
      nc.res_headers().as_slice(),
      middle,
      key,
      end,
    ])
    .await?;
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
) -> [(KnownHeaderName, Option<&'headers [u8]>); N] {
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
  rslt
}

fn derived_key<'buffer>(buffer: &'buffer mut [u8; 30], key: &[u8]) -> &'buffer [u8] {
  let array = Sha1HashGlobal::digest([key, b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"]);
  base64_from_array(&array, buffer)
}

fn gen_key<'buffer>(buffer: &'buffer mut [u8; 26], rng: &mut impl Rng) -> &'buffer [u8] {
  base64_from_array(&rng.u8_16(), buffer)
}

const fn has_no_masking(el: &[u8]) -> bool {
  el.eq_ignore_ascii_case(b"no-masking")
}

const fn no_masking_bytes(no_masking: bool) -> &'static [u8] {
  if no_masking { b"Sec-WebSocket-Extensions: no-masking\r\n" } else { &[] }
}
