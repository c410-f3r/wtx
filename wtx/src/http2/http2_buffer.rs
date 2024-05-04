use crate::{
  http2::{FrameInit, HpackDecoder, HpackEncoder, UriBuffer, U31},
  misc::{BlocksQueue, ByteVector, Lease, LeaseMut, PartitionedFilledBuffer},
  rng::Rng,
};
use alloc::boxed::Box;
use hashbrown::HashMap;

/// Groups all intermediate structures necessary to perform HTTP/2 connections.
//
// Maximum sizes are dictated by `AcceptParams` or `ConnectParams`.
#[derive(Debug)]
pub struct Http2Buffer<const IS_CLIENT: bool> {
  pub(crate) hpack_dec: HpackDecoder,
  pub(crate) hpack_enc: HpackEncoder,
  pub(crate) hpack_enc_buffer: ByteVector,
  pub(crate) pfb: PartitionedFilledBuffer,
  pub(crate) streams_frames: HashMap<U31, BlocksQueue<u8, FrameInit>>,
  pub(crate) uri_buffer: Box<UriBuffer>,
}

impl<const IS_CLIENT: bool> Http2Buffer<IS_CLIENT> {
  /// Creates a new instance without pre-allocated resources.
  #[inline]
  pub fn new<RNG>(rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      hpack_dec: HpackDecoder::new(),
      hpack_enc: HpackEncoder::new(rng),
      hpack_enc_buffer: ByteVector::new(),
      pfb: PartitionedFilledBuffer::new(),
      streams_frames: HashMap::new(),
      uri_buffer: Box::new(UriBuffer::new()),
    }
  }

  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { hpack_dec, hpack_enc, hpack_enc_buffer, pfb, streams_frames, uri_buffer } = self;
    hpack_dec.clear();
    hpack_enc.clear();
    hpack_enc_buffer.clear();
    pfb._clear();
    streams_frames.clear();
    uri_buffer.clear();
  }
}

impl<const IS_CLIENT: bool> Lease<Http2Buffer<IS_CLIENT>> for Http2Buffer<IS_CLIENT> {
  #[inline]
  fn lease(&self) -> &Http2Buffer<IS_CLIENT> {
    self
  }
}

impl<const IS_CLIENT: bool> LeaseMut<Http2Buffer<IS_CLIENT>> for Http2Buffer<IS_CLIENT> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Buffer<IS_CLIENT> {
    self
  }
}
