use crate::{
  collections::DequeU32,
  http2::{
    Scorp, Sovrp, hpack_decoder::HpackDecoder, hpack_encoder::HpackEncoder,
    initial_server_stream_remote::InitialServerStreamRemote,
  },
  net::BufStreamReader,
  rng::{Rng, Xorshift64, simple_seed},
};
use core::task::Waker;
use hashbrown::HashMap;

/// Groups all intermediate structures necessary to perform HTTP/2 connections.
#[derive(Debug)]
pub struct Http2Buffer {
  pub(crate) hpack_dec: HpackDecoder,
  pub(crate) hpack_enc: HpackEncoder,
  pub(crate) initial_server_streams_local: DequeU32<Waker>,
  pub(crate) initial_server_streams_remote: DequeU32<InitialServerStreamRemote>,
  pub(crate) nrb: BufStreamReader,
  pub(crate) scrps: Scorp,
  pub(crate) sorps: Sovrp,
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
      initial_server_streams_local: DequeU32::new(),
      initial_server_streams_remote: DequeU32::new(),
      nrb: BufStreamReader::new(),
      scrps: HashMap::new(),
      sorps: HashMap::new(),
    }
  }

  pub(crate) fn clear(&mut self) {
    let Self {
      hpack_dec,
      hpack_enc,
      initial_server_streams_local,
      initial_server_streams_remote,
      nrb,
      scrps,
      sorps,
    } = self;
    hpack_dec.clear();
    hpack_enc.clear();
    initial_server_streams_local.clear();
    initial_server_streams_remote.clear();
    nrb.clear();
    scrps.clear();
    sorps.clear();
  }

  pub(crate) fn drain_and_wake(&mut self) {
    while let Some(waker) = self.initial_server_streams_local.pop_front() {
      waker.wake();
    }
    for (_, value) in self.scrps.drain() {
      value.waker.wake();
    }
    for (_, value) in self.sorps.drain() {
      value.waker.wake();
    }
    self.initial_server_streams_remote.clear();
  }
}

impl Default for Http2Buffer {
  #[inline]
  fn default() -> Self {
    Self::new(&mut Xorshift64::from(simple_seed()))
  }
}
