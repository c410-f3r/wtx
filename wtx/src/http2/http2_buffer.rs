use core::task::Waker;

use crate::{
  collection::{Deque, Vector},
  http2::{
    Scrp, Sorp, hpack_decoder::HpackDecoder, hpack_encoder::HpackEncoder,
    initial_server_stream_remote::InitialServerStreamRemote,
  },
  misc::{Lease, LeaseMut, net::PartitionedFilledBuffer},
  rng::{Rng, Xorshift64, simple_seed},
};
use hashbrown::HashMap;

/// Groups all intermediate structures necessary to perform HTTP/2 connections.
#[derive(Debug)]
pub struct Http2Buffer {
  pub(crate) hpack_dec: HpackDecoder,
  pub(crate) hpack_enc_buffer: Vector<u8>,
  pub(crate) hpack_enc: HpackEncoder,
  pub(crate) initial_server_streams_local: Deque<Waker>,
  pub(crate) initial_server_streams_remote: Deque<InitialServerStreamRemote>,
  pub(crate) pfb: PartitionedFilledBuffer,
  pub(crate) scrp: Scrp,
  pub(crate) sorp: Sorp,
}

impl Http2Buffer {
  /// Creates a new instance without pre-allocated resources.
  #[inline]
  pub fn new<RNG>(rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      hpack_dec: HpackDecoder::new(),
      hpack_enc: HpackEncoder::new(rng),
      hpack_enc_buffer: Vector::new(),
      initial_server_streams_local: Deque::new(),
      initial_server_streams_remote: Deque::new(),
      pfb: PartitionedFilledBuffer::new(),
      scrp: HashMap::new(),
      sorp: HashMap::new(),
    }
  }

  pub(crate) fn clear(&mut self) {
    let Self {
      hpack_dec,
      hpack_enc,
      hpack_enc_buffer,
      initial_server_streams_local,
      initial_server_streams_remote,
      pfb,
      scrp,
      sorp,
    } = self;
    hpack_dec.clear();
    hpack_enc_buffer.clear();
    hpack_enc.clear();
    initial_server_streams_local.clear();
    initial_server_streams_remote.clear();
    pfb.clear();
    scrp.clear();
    sorp.clear();
  }
}

impl Default for Http2Buffer {
  #[inline]
  fn default() -> Self {
    Self::new(&mut Xorshift64::from(simple_seed()))
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
