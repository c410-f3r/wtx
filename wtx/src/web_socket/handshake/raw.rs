use crate::{
  http_structs::{Header, ParseStatus, Request, Response},
  misc::_trim,
  rng::Rng,
  web_socket::{
    compression::NegotiatedCompression,
    handshake::{
      misc::{derived_key, gen_key},
      HeadersBuffer, WebSocketAccept, WebSocketConnect,
    },
    Compression, FrameBuffer, WebSocketClient, WebSocketError, WebSocketServer,
  },
  ExpectedHeader, PartitionedBuffer, Stream, UriParts,
};
use core::{borrow::BorrowMut, str};

const MAX_READ_HEADER_LEN: usize = 64;
const MAX_READ_LEN: usize = 2 * 1024;

/// Marker used to implement [WebSocketAccept].
#[derive(Debug)]
pub struct WebSocketAcceptRaw<'any, C, PB, RNG, S> {
  /// Compression
  pub compression: C,
  /// Headers buffer
  pub headers_buffer: &'any mut HeadersBuffer<'any, 3>,
  /// Key buffer
  pub key_buffer: &'any mut [u8; 30],
  /// Partitioned buffer
  pub pb: PB,
  /// Random Number Generator
  pub rng: RNG,
  /// Stream
  pub stream: S,
}

impl<'any, C, PB, RNG, S> WebSocketAccept<C::Negotiated, PB, RNG>
  for WebSocketAcceptRaw<'any, C, PB, RNG, S>
where
  C: Compression<false>,
  PB: BorrowMut<PartitionedBuffer>,
  RNG: Rng,
  S: Stream,
{
  type Response = Response<'any, 'any>;
  type Stream = S;

  #[inline]
  async fn accept(
    mut self,
  ) -> crate::Result<(Self::Response, WebSocketServer<C::Negotiated, PB, RNG, Self::Stream>)> {
    let pb = self.pb.borrow_mut();
    pb._set_indices_through_expansion(0, 0, MAX_READ_LEN);
    let mut read = 0;
    loop {
      let read_buffer = pb._following_mut().get_mut(read..).unwrap_or_default();
      let local_read = self.stream.read(read_buffer).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedEOF);
      }
      read = read.wrapping_add(local_read);
      let mut req_buffer = [Header::EMPTY; MAX_READ_HEADER_LEN];
      let mut req = Request::new(&mut req_buffer);
      match req.parse(pb._following())? {
        ParseStatus::Complete(_) => {
          if !req.method().map_or(false, |el| _trim(el.as_bytes()).eq_ignore_ascii_case(b"get")) {
            return Err(crate::Error::UnexpectedHttpMethod);
          }
          verify_common_header(req.headers())?;
          if !has_header_key_and_value(req.headers(), "sec-websocket-version", b"13") {
            return Err(crate::Error::MissingHeader {
              expected: ExpectedHeader::SecWebSocketVersion_13,
            });
          };
          let Some(key) = req.headers().iter().find_map(|el| {
            (el.name().eq_ignore_ascii_case("sec-websocket-key")).then_some(el.value())
          }) else {
            return Err(crate::Error::MissingHeader { expected: ExpectedHeader::SecWebSocketKey });
          };
          let compression = self.compression.negotiate(req.headers())?;
          let swa = derived_key(self.key_buffer, key);
          self.headers_buffer.headers[0] = Header::new("Connection", b"Upgrade");
          self.headers_buffer.headers[1] = Header::new("Sec-WebSocket-Accept", swa.as_bytes());
          self.headers_buffer.headers[2] = Header::new("Upgrade", b"websocket");
          let mut res = Response::new(&mut self.headers_buffer.headers);
          *res.code_mut() = Some(101);
          *res.version_mut() = req.version();
          let res_bytes = build_res(&compression, res.headers(), pb);
          self.stream.write_all(res_bytes).await?;
          pb.clear();
          return Ok((res, WebSocketServer::new(compression, self.pb, self.rng, self.stream)));
        }
        ParseStatus::Partial => {}
      }
    }
  }
}

/// Marker used to implement [WebSocketConnect].
#[derive(Debug)]
pub struct WebSocketConnectRaw<'any, B, C, PB, RNG, S> {
  /// Initial compression
  pub compression: C,
  /// Frame buffer
  pub fb: &'any mut FrameBuffer<B>,
  /// Headers buffer
  pub headers_buffer: &'any mut HeadersBuffer<'any, MAX_READ_HEADER_LEN>,
  /// Partitioned Buffer
  pub pb: PB,
  /// Random Number Generator
  pub rng: RNG,
  /// Stream
  pub stream: S,
  /// Uri
  pub uri: &'any str,
}

impl<'any, B, C, PB, RNG, S> WebSocketConnect<C::Negotiated, PB, RNG>
  for WebSocketConnectRaw<'any, B, C, PB, RNG, S>
where
  B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
  C: Compression<true>,
  PB: BorrowMut<PartitionedBuffer>,
  RNG: Rng,
  S: Stream,
{
  type Response = Response<'any, 'any>;
  type Stream = S;

  #[inline]
  async fn connect(
    mut self,
  ) -> crate::Result<(Self::Response, WebSocketClient<C::Negotiated, PB, RNG, Self::Stream>)> {
    let key_buffer = &mut <_>::default();
    let pb = self.pb.borrow_mut();
    pb.clear();
    let (key, req) = build_req(&self.compression, key_buffer, pb, &mut self.rng, self.uri);
    self.stream.write_all(req).await?;
    let mut read = 0;
    self.fb._set_indices_through_expansion(0, 0, MAX_READ_LEN);
    let len = loop {
      let mut local_header = [Header::EMPTY; MAX_READ_HEADER_LEN];
      let read_buffer = self.fb.payload_mut().get_mut(read..).unwrap_or_default();
      let local_read = self.stream.read(read_buffer).await?;
      if local_read == 0 {
        return Err(crate::Error::UnexpectedEOF);
      }
      read = read.wrapping_add(local_read);
      match Response::new(&mut local_header).parse(self.fb.payload())? {
        ParseStatus::Complete(len) => break len,
        ParseStatus::Partial => {}
      }
    };
    let mut res = Response::new(&mut self.headers_buffer.headers);
    let _status = res.parse(self.fb.payload())?;
    if res.code() != Some(101) {
      return Err(WebSocketError::MissingSwitchingProtocols.into());
    }
    verify_common_header(res.headers())?;
    if !has_header_key_and_value(
      res.headers(),
      "sec-websocket-accept",
      derived_key(&mut <_>::default(), key.as_bytes()).as_bytes(),
    ) {
      return Err(crate::Error::MissingHeader { expected: crate::ExpectedHeader::SecWebSocketKey });
    }
    let compression = self.compression.negotiate(res.headers())?;
    pb.borrow_mut()._set_indices_through_expansion(0, 0, read.wrapping_sub(len));
    pb._following_mut().copy_from_slice(self.fb.payload().get(len..read).unwrap_or_default());
    Ok((res, WebSocketClient::new(compression, self.pb, self.rng, self.stream)))
  }
}

/// Client request
fn build_req<'pb, 'kb, C>(
  compression: &C,
  key_buffer: &'kb mut [u8; 26],
  pb: &'pb mut PartitionedBuffer,
  rng: &mut impl Rng,
  uri: &str,
) -> (&'kb str, &'pb [u8])
where
  C: Compression<true>,
{
  let uri_parts = UriParts::from(uri);
  let key = gen_key(key_buffer, rng);

  let idx = pb._buffer().len();
  pb.extend(b"GET ");
  pb.extend(uri_parts.href.as_bytes());
  pb.extend(b" HTTP/1.1\r\n");

  pb.extend(b"Connection: Upgrade\r\n");
  pb.extend(b"Host: ");
  pb.extend(uri_parts.host.as_bytes());
  pb.extend(b"\r\n");
  pb.extend(b"Sec-WebSocket-Key: ");
  pb.extend(key.as_bytes());
  pb.extend(b"\r\n");
  pb.extend(b"Sec-WebSocket-Version: 13\r\n");
  pb.extend(b"Upgrade: websocket\r\n");

  compression.write_req_headers(pb);

  pb.extend(b"\r\n");

  (key, pb._buffer().get(idx..).unwrap_or_default())
}

/// Server response
fn build_res<'pb, C>(
  compression: &C,
  headers: &[Header<'_>],
  pb: &'pb mut PartitionedBuffer,
) -> &'pb [u8]
where
  C: NegotiatedCompression,
{
  let idx = pb._buffer().len();
  pb.extend(b"HTTP/1.1 101 Switching Protocols\r\n");
  for header in headers {
    pb.extend(header.name().as_bytes());
    pb.extend(b": ");
    pb.extend(header.value());
    pb.extend(b"\r\n");
  }
  compression.write_res_headers(pb);
  pb.extend(b"\r\n");
  pb._buffer().get(idx..).unwrap_or_default()
}

fn has_header_key_and_value(headers: &[Header<'_>], key: &str, value: &[u8]) -> bool {
  headers
    .iter()
    .find_map(|h| {
      let has_key = _trim(h.name().as_bytes()).eq_ignore_ascii_case(key.as_bytes());
      let has_value =
        h.value().split(|el| el == &b',').any(|el| _trim(el).eq_ignore_ascii_case(value));
      (has_key && has_value).then_some(true)
    })
    .unwrap_or(false)
}

fn verify_common_header(buffer: &[Header<'_>]) -> crate::Result<()> {
  if !has_header_key_and_value(buffer, "connection", b"upgrade") {
    return Err(crate::Error::MissingHeader { expected: ExpectedHeader::Connection_Upgrade });
  }
  if !has_header_key_and_value(buffer, "upgrade", b"websocket") {
    return Err(crate::Error::MissingHeader { expected: ExpectedHeader::Upgrade_WebSocket });
  }
  Ok(())
}
