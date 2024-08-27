use crate::{
  http::Method,
  http2::{HpackDecoder, HpackEncoder, Scrp, Sorp, UriBuffer, U31},
  misc::{AtomicWaker, Lease, LeaseMut, PartitionedFilledBuffer, Rng, Vector},
};
use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
use core::{
  sync::atomic::{AtomicBool, Ordering},
  task::Waker,
};
use hashbrown::HashMap;

/// Groups all intermediate structures necessary to perform HTTP/2 connections.
#[derive(Debug)]
pub struct Http2Buffer<RRB> {
  pub(crate) hpack_dec: HpackDecoder,
  pub(crate) hpack_enc: HpackEncoder,
  pub(crate) hpack_enc_buffer: Vector<u8>,
  pub(crate) initial_server_header_buffers: VecDeque<(RRB, Waker)>,
  pub(crate) initial_server_header_params: VecDeque<(Method, U31)>,
  pub(crate) is_conn_open: Arc<AtomicBool>,
  pub(crate) pfb: PartitionedFilledBuffer,
  pub(crate) read_frame_waker: Arc<AtomicWaker>,
  pub(crate) scrp: Scrp,
  pub(crate) sorp: Sorp<RRB>,
  pub(crate) uri_buffer: Box<UriBuffer>,
}

impl<RRB> Http2Buffer<RRB> {
  /// Creates a new instance without pre-allocated resources.
  #[inline]
  pub fn new<RNG>(rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      hpack_dec: HpackDecoder::new(),
      hpack_enc: HpackEncoder::new(rng),
      hpack_enc_buffer: Vector::new(),
      initial_server_header_buffers: VecDeque::new(),
      initial_server_header_params: VecDeque::new(),
      is_conn_open: Arc::new(AtomicBool::new(false)),
      pfb: PartitionedFilledBuffer::new(),
      read_frame_waker: Arc::new(AtomicWaker::new()),
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
      hpack_enc_buffer,
      initial_server_header_buffers,
      initial_server_header_params,
      is_conn_open,
      pfb,
      read_frame_waker,
      scrp,
      sorp,
      uri_buffer,
    } = self;
    hpack_dec.clear();
    hpack_enc.clear();
    hpack_enc_buffer.clear();
    initial_server_header_buffers.clear();
    initial_server_header_params.clear();
    is_conn_open.store(false, Ordering::Relaxed);
    pfb._clear();
    let _waker = read_frame_waker.take();
    scrp.clear();
    sorp.clear();
    uri_buffer.clear();
  }
}

#[cfg(feature = "std")]
impl<RRB> Default for Http2Buffer<RRB> {
  #[inline]
  fn default() -> Self {
    Self::new(crate::misc::StdRng::default())
  }
}

impl<RRB> Lease<Http2Buffer<RRB>> for Http2Buffer<RRB> {
  #[inline]
  fn lease(&self) -> &Http2Buffer<RRB> {
    self
  }
}

impl<RRB> LeaseMut<Http2Buffer<RRB>> for Http2Buffer<RRB> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Http2Buffer<RRB> {
    self
  }
}
