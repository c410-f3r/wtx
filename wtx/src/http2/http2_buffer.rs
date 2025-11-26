use crate::{
  collection::{Deque, Vector},
  http2::{
    Scrp, Sorp, hpack_decoder::HpackDecoder, hpack_encoder::HpackEncoder,
    local_server_stream::LocalServerStream,
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
  pub(crate) is_conn_open: Arc<AtomicBool>,
  pub(crate) local_server_streams: Deque<LocalServerStream>,
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
      is_conn_open: Arc::new(AtomicBool::new(false)),
      local_server_streams: Deque::new(),
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
      is_conn_open,
      local_server_streams,
      pfb,
      read_frame_waker,
      scrp,
      sorp,
    } = self;
    hpack_dec.clear();
    hpack_enc.clear();
    hpack_enc_buffer.clear();
    is_conn_open.store(false, Ordering::Relaxed);
    local_server_streams.clear();
    pfb.clear();
    let _read_frame_waker = read_frame_waker.take();
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
