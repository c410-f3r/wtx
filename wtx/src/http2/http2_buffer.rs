use crate::{
  collection::{IndexedStorageMut, Vector},
  http2::{
    Scrp, Sorp, hpack_decoder::HpackDecoder, hpack_encoder::HpackEncoder, index_map::IndexMap,
    initial_server_header::InitialServerHeader,
  },
  misc::{Lease, LeaseMut, net::PartitionedFilledBuffer},
  rng::{Rng, Xorshift64, simple_seed},
  sync::{Arc, AtomicBool, AtomicWaker},
};
use core::sync::atomic::Ordering;
use hashbrown::HashMap;

/// Groups all intermediate structures necessary to perform HTTP/2 connections.
#[derive(Debug)]
pub struct Http2Buffer {
  pub(crate) hpack_dec: HpackDecoder,
  pub(crate) hpack_enc: HpackEncoder,
  pub(crate) hpack_enc_buffer: Vector<u8>,
  pub(crate) initial_server_headers: IndexMap<u32, InitialServerHeader>,
  pub(crate) is_conn_open: Arc<AtomicBool>,
  pub(crate) pfb: PartitionedFilledBuffer,
  pub(crate) read_frame_waker: Arc<AtomicWaker>,
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
      initial_server_headers: IndexMap::new(),
      is_conn_open: Arc::new(AtomicBool::new(false)),
      pfb: PartitionedFilledBuffer::new(),
      read_frame_waker: Arc::new(AtomicWaker::new()),
      scrp: HashMap::new(),
      sorp: HashMap::new(),
    }
  }

  pub(crate) fn clear(&mut self) {
    let Self {
      hpack_dec,
      hpack_enc,
      hpack_enc_buffer,
      initial_server_headers,
      is_conn_open,
      pfb,
      read_frame_waker,
      scrp,
      sorp,
    } = self;
    hpack_dec.clear();
    hpack_enc.clear();
    hpack_enc_buffer.clear();
    initial_server_headers.clear();
    is_conn_open.store(false, Ordering::Relaxed);
    pfb.clear();
    let _waker = read_frame_waker.take();
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
