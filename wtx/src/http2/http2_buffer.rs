use crate::{
  http2::{HpackDecoder, HpackEncoder, StreamData, StreamId, UriBuffer},
  misc::{ByteVector, PartitionedFilledBuffer},
  rng::Rng,
};
use alloc::boxed::Box;
use hashbrown::HashMap;

#[derive(Debug)]
pub struct Http2Buffer<const IS_CLIENT: bool> {
  pub(crate) hpack_dec: HpackDecoder,
  pub(crate) hpack_enc: HpackEncoder,
  pub(crate) rb: PartitionedFilledBuffer,
  pub(crate) streams_data: HashMap<StreamId, StreamData<IS_CLIENT>>,
  pub(crate) uri_buffer: Box<UriBuffer>,
  pub(crate) wb: ByteVector,
}

impl<const IS_CLIENT: bool> Http2Buffer<IS_CLIENT> {
  pub fn with_capacity<RNG>(rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      hpack_dec: HpackDecoder::with_capacity(0, 0, 0),
      hpack_enc: HpackEncoder::with_capacity(0, 0, 0, rng),
      rb: PartitionedFilledBuffer::with_capacity(0),
      streams_data: HashMap::with_capacity(0),
      uri_buffer: Box::default(),
      wb: ByteVector::with_capacity(0),
    }
  }

  pub(crate) fn clear(&mut self) {
    let Self { hpack_dec, hpack_enc, rb, streams_data, uri_buffer, wb } = self;
    hpack_dec._clear();
    hpack_enc._clear();
    rb._clear();
    streams_data.clear();
    uri_buffer.clear();
    wb.clear();
  }
}
