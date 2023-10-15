use crate::web_socket::{handshake::HeadersBuffer, FrameBuffer};

const MAX_READ_HEADER_LEN: usize = 64;

/// Marker used to implement [WebSocketAccept].
#[derive(Debug)]
pub struct WebSocketAcceptRaw<'kb, C, PB, RNG, S> {
  /// Compression
  pub compression: C,
  /// Key buffer
  pub key_buffer: &'kb mut [u8; 30],
  /// Partitioned buffer
  pub pb: PB,
  /// Random Number Generator
  pub rng: RNG,
  /// Stream
  pub stream: S,
}

/// Marker used to implement [WebSocketConnect].
#[derive(Debug)]
pub struct WebSocketConnectRaw<'fb, 'hb, 'uri, B, C, H, PB, RNG, S> {
  /// Initial compression
  pub compression: C,
  /// Frame buffer
  pub fb: &'fb mut FrameBuffer<B>,
  /// Headers buffer
  pub headers_buffer: &'hb mut HeadersBuffer<H, MAX_READ_HEADER_LEN>,
  /// Partitioned Buffer
  pub pb: PB,
  /// Random Number Generator
  pub rng: RNG,
  /// Stream
  pub stream: S,
  /// Uri
  pub uri: &'uri str,
}

#[cfg(feature = "web-socket-handshake")]
mod httparse_impls {
  use crate::{
    http::{Header as _, Request as _},
    misc::_trim,
    rng::Rng,
    web_socket::{
      compression::NegotiatedCompression,
      handshake::{
        misc::{derived_key, gen_key},
        raw::MAX_READ_HEADER_LEN,
        HeadersBuffer, WebSocketAccept, WebSocketAcceptRaw, WebSocketConnect, WebSocketConnectRaw,
      },
      Compression, WebSocketClient, WebSocketError, WebSocketServer,
    },
    AsyncBounds, ExpectedHeader, PartitionedBuffer, Stream, UriParts,
  };
  use alloc::vec::Vec;
  use core::{borrow::BorrowMut, str};
  use httparse::{Header, Request, Response, Status, EMPTY_HEADER};

  const MAX_READ_LEN: usize = 2 * 1024;

  impl<'kb, C, PB, RNG, S> WebSocketAccept<C::NegotiatedCompression, PB, RNG, S>
    for WebSocketAcceptRaw<'kb, C, PB, RNG, S>
  where
    C: AsyncBounds + Compression<false>,
    C::NegotiatedCompression: AsyncBounds,
    PB: AsyncBounds + BorrowMut<PartitionedBuffer>,
    RNG: AsyncBounds + Rng,
    S: AsyncBounds + Stream,
  {
    #[inline]
    async fn accept(
      mut self,
    ) -> crate::Result<WebSocketServer<C::NegotiatedCompression, PB, RNG, S>> {
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
        let mut req_buffer = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
        let mut req = Request::new(&mut req_buffer);
        match req.parse(pb._following())? {
          Status::Complete(_) => {
            if !_trim(req.method()).eq_ignore_ascii_case(b"get") {
              return Err(crate::Error::UnexpectedHttpMethod);
            }
            verify_common_header(req.headers)?;
            if !has_header_key_and_value(req.headers, b"sec-websocket-version", b"13") {
              return Err(crate::Error::MissingHeader {
                expected: ExpectedHeader::SecWebSocketVersion_13,
              });
            };
            let Some(key) = req.headers.iter().find_map(|el| {
              (el.name().eq_ignore_ascii_case(b"sec-websocket-key")).then_some(el.value())
            }) else {
              return Err(crate::Error::MissingHeader {
                expected: ExpectedHeader::SecWebSocketKey,
              });
            };
            let compression = self.compression.negotiate(req.headers.iter())?;
            let swa = derived_key(self.key_buffer, key);
            let mut headers_buffer = HeadersBuffer::<_, 3>::default();
            headers_buffer.headers[0] = Header { name: "Connection", value: b"Upgrade" };
            headers_buffer.headers[1] = Header { name: "Sec-WebSocket-Accept", value: swa };
            headers_buffer.headers[2] = Header { name: "Upgrade", value: b"websocket" };
            let mut res = Response::new(&mut headers_buffer.headers);
            res.code = Some(101);
            res.version = Some(req.version().into());
            let res_bytes = build_res(&compression, res.headers, pb);
            self.stream.write_all(res_bytes).await?;
            pb.clear();
            return Ok(WebSocketServer::new(compression, self.pb, self.rng, self.stream));
          }
          Status::Partial => {}
        }
      }
    }
  }

  impl<'fb, 'hb, 'uri, B, C, PB, RNG, S> WebSocketConnect<C::NegotiatedCompression, PB, RNG, S>
    for WebSocketConnectRaw<'fb, 'hb, 'uri, B, C, Header<'fb>, PB, RNG, S>
  where
    B: AsyncBounds + AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
    C: AsyncBounds + Compression<true>,
    C::NegotiatedCompression: AsyncBounds,
    PB: AsyncBounds + BorrowMut<PartitionedBuffer>,
    RNG: AsyncBounds + Rng,
    S: AsyncBounds + Stream,
    'fb: 'hb,
  {
    type Response = Response<'hb, 'fb>;

    #[inline]
    async fn connect(
      mut self,
    ) -> crate::Result<(Self::Response, WebSocketClient<C::NegotiatedCompression, PB, RNG, S>)>
    {
      let key_buffer = &mut <_>::default();
      let pb = self.pb.borrow_mut();
      pb.clear();
      let (key, req) = build_req(&self.compression, key_buffer, pb, &mut self.rng, self.uri);
      self.stream.write_all(req).await?;
      let mut read = 0;
      self.fb._set_indices_through_expansion(0, 0, MAX_READ_LEN);
      let len = loop {
        let mut local_header = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
        let read_buffer = self.fb.payload_mut().get_mut(read..).unwrap_or_default();
        let local_read = self.stream.read(read_buffer).await?;
        if local_read == 0 {
          return Err(crate::Error::UnexpectedEOF);
        }
        read = read.wrapping_add(local_read);
        match Response::new(&mut local_header).parse(self.fb.payload())? {
          Status::Complete(len) => break len,
          Status::Partial => {}
        }
      };
      let mut res = Response::new(&mut self.headers_buffer.headers);
      let _status = res.parse(self.fb.payload())?;
      if res.code != Some(101) {
        return Err(WebSocketError::MissingSwitchingProtocols.into());
      }
      verify_common_header(res.headers)?;
      if !has_header_key_and_value(
        res.headers,
        b"sec-websocket-accept",
        derived_key(&mut <_>::default(), key),
      ) {
        return Err(crate::Error::MissingHeader {
          expected: crate::ExpectedHeader::SecWebSocketKey,
        });
      }
      let compression = self.compression.negotiate(res.headers.iter())?;
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
  ) -> (&'kb [u8], &'pb [u8])
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
    pb.extend(key);
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
      pb.extend(header.name());
      pb.extend(b": ");
      pb.extend(header.value());
      pb.extend(b"\r\n");
    }
    compression.write_res_headers(pb);
    pb.extend(b"\r\n");
    pb._buffer().get(idx..).unwrap_or_default()
  }

  fn has_header_key_and_value(headers: &[Header<'_>], key: &[u8], value: &[u8]) -> bool {
    headers
      .iter()
      .find_map(|h| {
        let has_key = _trim(h.name()).eq_ignore_ascii_case(key);
        let has_value =
          h.value().split(|el| el == &b',').any(|el| _trim(el).eq_ignore_ascii_case(value));
        (has_key && has_value).then_some(true)
      })
      .unwrap_or(false)
  }

  fn verify_common_header(buffer: &[Header<'_>]) -> crate::Result<()> {
    if !has_header_key_and_value(buffer, b"connection", b"upgrade") {
      return Err(crate::Error::MissingHeader { expected: ExpectedHeader::Connection_Upgrade });
    }
    if !has_header_key_and_value(buffer, b"upgrade", b"websocket") {
      return Err(crate::Error::MissingHeader { expected: ExpectedHeader::Upgrade_WebSocket });
    }
    Ok(())
  }
}
