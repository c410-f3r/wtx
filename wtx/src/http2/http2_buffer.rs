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
pub struct Http2Buffer {
  pub(crate) hpack_dec: HpackDecoder,
  pub(crate) hpack_enc: HpackEncoder,
  pub(crate) hpack_enc_buffer: ByteVector,
  pub(crate) pfb: PartitionedFilledBuffer,
  pub(crate) streams_frames: HashMap<U31, BlocksQueue<u8, FrameInit>>,
  pub(crate) uri_buffer: Box<UriBuffer>,
}

impl Http2Buffer {
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

impl Lease<Http2Buffer> for Http2Buffer {
  #[inline]
  fn lease(&self) -> &Http2Buffer {
    self
  }
}

impl LeaseMut<Http2Buffer> for Http2Buffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Buffer {
    self
  }
}
