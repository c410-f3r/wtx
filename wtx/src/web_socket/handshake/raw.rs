use crate::{
  misc::UriRef,
  web_socket::{handshake::HeadersBuffer, FrameBuffer},
};

const MAX_READ_HEADER_LEN: usize = 64;

/// Marker used to implement `WebSocketAccept`.
#[derive(Debug)]
pub struct WebSocketAcceptRaw<C, RNG, S, WSB> {
  /// Compression
  pub compression: C,
  /// Random Number Generator
  pub rng: RNG,
  /// Stream
  pub stream: S,
  /// WebSocket Buffer
  pub wsb: WSB,
}

/// Marker used to implement `WebSocketConnect`.
#[derive(Debug)]
pub struct WebSocketConnectRaw<'fb, 'hb, 'uri, B, C, H, RNG, S, WSB> {
  /// Initial compression
  pub compression: C,
  /// Frame buffer
  pub fb: &'fb mut FrameBuffer<B>,
  /// Headers buffer
  pub headers_buffer: &'hb mut HeadersBuffer<H, MAX_READ_HEADER_LEN>,
  /// Random Number Generator
  pub rng: RNG,
  /// Stream
  pub stream: S,
  /// Uri
  pub uri: &'uri UriRef<'uri>,
  /// WebSocket Buffer
  pub wsb: WSB,
}

#[cfg(feature = "web-socket-handshake")]
mod httparse_impls {
  use crate::{
    http::{ExpectedHeader, Header as _, Request as _},
    misc::{FilledBufferWriter, Stream, UriRef},
    rng::Rng,
    web_socket::{
      compression::NegotiatedCompression,
      handshake::{
        misc::{derived_key, gen_key},
        raw::MAX_READ_HEADER_LEN,
        HeadersBuffer, WebSocketAccept, WebSocketAcceptRaw, WebSocketConnect, WebSocketConnectRaw,
      },
      misc::_trim_bytes,
      Compression, WebSocketBuffer, WebSocketClient, WebSocketServer,
    },
  };
  use alloc::vec::Vec;
  use core::borrow::BorrowMut;
  use httparse::{Header, Request, Response, Status, EMPTY_HEADER};

  const MAX_READ_LEN: usize = 2 * 1024;

  impl<C, RNG, S, WSB> WebSocketAccept<C::NegotiatedCompression, RNG, S, WSB>
    for WebSocketAcceptRaw<C, RNG, S, WSB>
  where
    C: Compression<false>,
    RNG: Rng,
    S: Stream,
    WSB: BorrowMut<WebSocketBuffer>,
  {
    #[inline]
    async fn accept(
      mut self,
      cb: impl FnOnce(&dyn crate::http::Request) -> bool,
    ) -> crate::Result<WebSocketServer<C::NegotiatedCompression, RNG, S, WSB>> {
      let nb = &mut self.wsb.borrow_mut().nb;
      nb._set_indices_through_expansion(0, 0, MAX_READ_LEN);
      let mut read = 0;
      loop {
        let read_buffer = nb._following_mut().get_mut(read..).unwrap_or_default();
        let local_read = self.stream.read(read_buffer).await?;
        if local_read == 0 {
          return Err(crate::Error::UnexpectedEOF);
        }
        read = read.wrapping_add(local_read);
        let mut req_buffer = [EMPTY_HEADER; MAX_READ_HEADER_LEN];
        let mut req = Request::new(&mut req_buffer);
        match req.parse(nb._following())? {
          Status::Complete(_) => {
            if !cb(&req) {
              return Err(crate::Error::InvalidAcceptRequest);
            }
            if !_trim_bytes(req.method()).eq_ignore_ascii_case(b"get") {
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
            let mut key_buffer = [0; 30];
            let swa = derived_key(&mut key_buffer, key);
            let mut headers_buffer = HeadersBuffer::<_, 3>::default();
            headers_buffer.headers[0] = Header { name: "Connection", value: b"Upgrade" };
            headers_buffer.headers[1] = Header { name: "Sec-WebSocket-Accept", value: swa };
            headers_buffer.headers[2] = Header { name: "Upgrade", value: b"websocket" };
            let mut res = Response::new(&mut headers_buffer.headers);
            res.code = Some(101);
            res.version = Some(req.version().into());
            let mut fbw = nb.into();
            let res_bytes = build_res(&compression, &mut fbw, res.headers);
            self.stream.write_all(res_bytes).await?;
            nb._clear();
            return Ok(WebSocketServer::new(compression, self.rng, self.stream, self.wsb));
          }
          Status::Partial => {}
        }
      }
    }
  }

  impl<'fb, 'hb, B, C, RNG, S, WSB> WebSocketConnect<C::NegotiatedCompression, RNG, S, WSB>
    for WebSocketConnectRaw<'fb, 'hb, '_, B, C, Header<'fb>, RNG, S, WSB>
  where
    B: AsMut<[u8]> + AsMut<Vec<u8>> + AsRef<[u8]>,
    C: Compression<true>,
    RNG: Rng,
    S: Stream,
    WSB: BorrowMut<WebSocketBuffer>,
    'fb: 'hb,
  {
    type Response = Response<'hb, 'fb>;

    #[inline]
    async fn connect(
      mut self,
    ) -> crate::Result<(Self::Response, WebSocketClient<C::NegotiatedCompression, RNG, S, WSB>)>
    {
      let key_buffer = &mut <_>::default();
      let nb = &mut self.wsb.borrow_mut().nb;
      nb._clear();
      let mut fbw = nb.into();
      let key = build_req(&self.compression, &mut fbw, key_buffer, &mut self.rng, self.uri);
      self.stream.write_all(fbw._curr_bytes()).await?;
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
        return Err(crate::Error::MissingSwitchingProtocols);
      }
      verify_common_header(res.headers)?;
      if !has_header_key_and_value(
        res.headers,
        b"sec-websocket-accept",
        derived_key(&mut <_>::default(), key),
      ) {
        return Err(crate::Error::MissingHeader {
          expected: crate::http::ExpectedHeader::SecWebSocketKey,
        });
      }
      let compression = self.compression.negotiate(res.headers.iter())?;
      nb.borrow_mut()._set_indices_through_expansion(0, 0, read.wrapping_sub(len));
      nb._following_mut().copy_from_slice(self.fb.payload().get(len..read).unwrap_or_default());
      Ok((res, WebSocketClient::new(compression, self.rng, self.stream, self.wsb)))
    }
  }

  /// Client request
  fn build_req<'kb, C>(
    compression: &C,
    fbw: &mut FilledBufferWriter<'_>,
    key_buffer: &'kb mut [u8; 26],
    rng: &mut impl Rng,
    uri: &UriRef<'_>,
  ) -> &'kb [u8]
  where
    C: Compression<true>,
  {
    let key = gen_key(key_buffer, rng);
    fbw._extend_from_slices_group_rn(&[b"GET ", uri.href().as_bytes(), b" HTTP/1.1"]);
    fbw._extend_from_slice_rn(b"Connection: Upgrade");
    match (uri.schema(), uri.port()) {
      ("http" | "ws", "80") | ("https" | "wss", "443") => {
        fbw._extend_from_slices_group_rn(&[b"Host: ", uri.hostname().as_bytes()]);
      }
      _ => fbw._extend_from_slices_group_rn(&[b"Host: ", uri.host().as_bytes()]),
    }
    fbw._extend_from_slices_group_rn(&[b"Sec-WebSocket-Key: ", key]);
    fbw._extend_from_slice_rn(b"Sec-WebSocket-Version: 13");
    fbw._extend_from_slice_rn(b"Upgrade: websocket");
    compression.write_req_headers(fbw);
    fbw._extend_from_slice_rn(b"");
    key
  }

  /// Server response
  fn build_res<'fpb, C>(
    compression: &C,
    fbw: &'fpb mut FilledBufferWriter<'fpb>,
    headers: &[Header<'_>],
  ) -> &'fpb [u8]
  where
    C: NegotiatedCompression,
  {
    fbw._extend_from_slice_rn(b"HTTP/1.1 101 Switching Protocols");
    for header in headers {
      fbw._extend_from_slices_group_rn(&[header.name(), b": ", header.value()]);
    }
    compression.write_res_headers(fbw);
    fbw._extend_from_slice_rn(b"");
    fbw._curr_bytes()
  }

  fn has_header_key_and_value(headers: &[Header<'_>], key: &[u8], value: &[u8]) -> bool {
    headers
      .iter()
      .find_map(|h| {
        let has_key = _trim_bytes(h.name()).eq_ignore_ascii_case(key);
        let has_value =
          h.value().split(|el| el == &b',').any(|el| _trim_bytes(el).eq_ignore_ascii_case(value));
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
