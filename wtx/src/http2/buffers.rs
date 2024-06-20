use crate::{
  http::{Headers, Method, Request, Response, StatusCode, Version},
  http2::{uri_buffer::MAX_URI_LEN, HpackDecoder, HpackEncoder, Scrp, Sorp, UriBuffer, U31},
  misc::{ArrayString, ByteVector, Lease, LeaseMut, PartitionedFilledBuffer, Queue, UriRef},
  rng::Rng,
};
use alloc::{boxed::Box, vec::Vec};
use hashbrown::HashMap;

/// Groups all intermediate structures necessary to perform HTTP/2 connections.
#[derive(Debug)]
pub struct Http2Buffer<SB> {
  pub(crate) hpack_dec: HpackDecoder,
  pub(crate) hpack_enc: HpackEncoder,
  pub(crate) initial_server_buffers: Vec<SB>,
  pub(crate) initial_server_streams: Queue<(Method, U31)>,
  pub(crate) pfb: PartitionedFilledBuffer,
  pub(crate) scrp: Scrp,
  pub(crate) sorp: Sorp<SB>,
  pub(crate) uri_buffer: Box<UriBuffer>,
}

impl<SB> Http2Buffer<SB> {
  /// Creates a new instance without pre-allocated resources.
  #[inline]
  pub fn new<RNG>(rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      hpack_dec: HpackDecoder::new(),
      hpack_enc: HpackEncoder::new(rng),
      initial_server_buffers: Vec::new(),
      initial_server_streams: Queue::new(),
      pfb: PartitionedFilledBuffer::new(),
      scrp: HashMap::new(),
      sorp: HashMap::new(),
      uri_buffer: Box::new(UriBuffer::new()),
    }
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self {
      hpack_dec,
      hpack_enc,
      initial_server_buffers,
      initial_server_streams,
      pfb,
      scrp,
      sorp,
      uri_buffer,
    } = self;
    hpack_dec.clear();
    hpack_enc.clear();
    initial_server_buffers.clear();
    initial_server_streams.clear();
    pfb._clear();
    scrp.clear();
    sorp.clear();
    uri_buffer.clear();
  }
}

impl<SB> Lease<Http2Buffer<SB>> for Http2Buffer<SB> {
  #[inline]
  fn lease(&self) -> &Http2Buffer<SB> {
    self
  }
}

impl<SB> LeaseMut<Http2Buffer<SB>> for Http2Buffer<SB> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Buffer<SB> {
    self
  }
}

/// Buffer used for requests or responses.
#[derive(Debug)]
pub struct ReqResBuffer {
  /// For sending and receiving body data.
  pub body: ByteVector,
  /// For sending and receiving headers.
  pub headers: Headers,
  /// Scheme, authority and path.
  pub uri: ArrayString<{ MAX_URI_LEN }>,
}

impl ReqResBuffer {
  /// Shortcut to create a HTTP/2 [Request] with inner referenced data.
  #[inline]
  pub fn as_http2_request(&self, method: Method) -> Request<(&ByteVector, &Headers), &str> {
    Request {
      data: (&self.body, &self.headers),
      method,
      uri: UriRef::new(self.uri.as_str()),
      version: Version::Http2,
    }
  }

  /// Mutable version of [`Self::as_http2_request`].
  #[inline]
  pub fn as_http2_request_mut(
    &mut self,
    method: Method,
  ) -> Request<(&mut ByteVector, &mut Headers), &str> {
    Request {
      data: (&mut self.body, &mut self.headers),
      method,
      uri: UriRef::new(self.uri.as_str()),
      version: Version::Http2,
    }
  }

  /// Shortcut to create a HTTP/2 [Response] with inner referenced data.
  #[inline]
  pub fn as_http2_response(&self, status_code: StatusCode) -> Response<(&ByteVector, &Headers)> {
    Response { data: (&self.body, &self.headers), status_code, version: Version::Http2 }
  }

  /// Mutable version of [`Self::as_http2_response`].
  #[inline]
  pub fn as_http2_response_mut(
    &mut self,
    status_code: StatusCode,
  ) -> Response<(&mut ByteVector, &mut Headers)> {
    Response { data: (&mut self.body, &mut self.headers), status_code, version: Version::Http2 }
  }
}

/// Buffer used for incoming or outgoing streams.
///
/// Internal operations automatically allocate memory but manual insertions should probably be
/// preceded with `reserve` calls.
#[derive(Debug)]
pub struct StreamBuffer {
  /// Hpack encoding buffer
  pub hpack_enc_buffer: ByteVector,
  /// See [ReqResBuffer].
  pub rrb: ReqResBuffer,
}

impl StreamBuffer {
  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { hpack_enc_buffer, rrb: ReqResBuffer { body, headers, uri } } = self;
    body.clear();
    headers.clear();
    hpack_enc_buffer.clear();
    uri.clear();
  }
}

// For servers, the default headers length must be used until a settings frame is received.
impl Default for StreamBuffer {
  #[inline]
  fn default() -> Self {
    Self {
      hpack_enc_buffer: ByteVector::new(),
      rrb: ReqResBuffer {
        body: ByteVector::new(),
        headers: Headers::new(0),
        uri: ArrayString::new(),
      },
    }
  }
}

impl Lease<StreamBuffer> for StreamBuffer {
  #[inline]
  fn lease(&self) -> &StreamBuffer {
    self
  }
}

impl LeaseMut<StreamBuffer> for StreamBuffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut StreamBuffer {
    self
  }
}
