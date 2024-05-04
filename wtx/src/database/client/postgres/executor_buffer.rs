use crate::{
  database::{
    client::postgres::{ty::Ty, Statements},
    Identifier,
  },
  misc::{Lease, LeaseMut, PartitionedFilledBuffer},
  rng::Rng,
};
use alloc::vec::Vec;
use core::ops::Range;
use hashbrown::HashMap;

pub(crate) const DFLT_PARAMS_LEN: usize = 16;
pub(crate) const DFLT_RECORDS_LEN: usize = 32;
pub(crate) const DFLT_VALUES_LEN: usize = 16;

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct ExecutorBuffer {
  /// Fetch type buffer
  pub(crate) ftb: Vec<(usize, u32)>,
  /// Network Buffer.
  pub(crate) nb: PartitionedFilledBuffer,
  /// Asynchronous parameters received from the database.
  pub(crate) params: Vec<(Identifier, Identifier)>,
  /// Records Buffer.
  pub(crate) rb: Vec<usize>,
  /// Statements
  pub(crate) stmts: Statements,
  /// Types buffer
  pub(crate) tb: HashMap<u32, Ty>,
  /// Values Buffer.
  pub(crate) vb: Vec<(bool, Range<usize>)>,
}

impl ExecutorBuffer {
  /// With provided capacity.
  #[inline]
  pub fn new<RNG>(
    (network_buffer_cap, records_buffer_cap, values_buffer_cap): (usize, usize, usize),
    rng: &mut RNG,
    max_queries: usize,
  ) -> Self
  where
    RNG: Rng,
  {
    Self {
      ftb: Vec::new(),
      nb: PartitionedFilledBuffer::_with_capacity(network_buffer_cap),
      params: Vec::with_capacity(DFLT_PARAMS_LEN),
      rb: Vec::with_capacity(records_buffer_cap),
      stmts: Statements::new(max_queries, rng),
      tb: HashMap::new(),
      vb: Vec::with_capacity(values_buffer_cap),
    }
  }

  /// With default capacity.
  #[inline]
  pub fn with_default_params<RNG>(rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      ftb: Vec::new(),
      nb: PartitionedFilledBuffer::default(),
      params: Vec::with_capacity(DFLT_PARAMS_LEN),
      rb: Vec::with_capacity(DFLT_RECORDS_LEN),
      stmts: Statements::with_default_params(rng),
      tb: HashMap::new(),
      vb: Vec::with_capacity(DFLT_VALUES_LEN),
    }
  }

  pub(crate) fn _empty() -> Self {
    Self {
      ftb: Vec::new(),
      nb: PartitionedFilledBuffer::new(),
      params: Vec::new(),
      rb: Vec::new(),
      stmts: Statements::_empty(),
      tb: HashMap::new(),
      vb: Vec::new(),
    }
  }

  /// Should be used in a new instance.
  pub(crate) fn clear(&mut self) {
    let Self { ftb, nb, params, rb, stmts, tb, vb } = self;
    ftb.clear();
    nb._clear();
    params.clear();
    rb.clear();
    stmts.clear();
    tb.clear();
    vb.clear();
  }

  /// Should be called before executing commands.
  pub(crate) fn clear_cmd_buffers(
    nb: &mut PartitionedFilledBuffer,
    rb: &mut Vec<usize>,
    vb: &mut Vec<(bool, Range<usize>)>,
  ) {
    nb._clear_if_following_is_empty();
    rb.clear();
    vb.clear();
  }

  pub(crate) fn parts_mut(&mut self) -> ExecutorBufferPartsMut<'_> {
    ExecutorBufferPartsMut {
      nb: &mut self.nb,
      params: &mut self.params,
      rb: &mut self.rb,
      stmts: &mut self.stmts,
      vb: &mut self.vb,
    }
  }
}

impl Lease<ExecutorBuffer> for ExecutorBuffer {
  #[inline]
  fn lease(&self) -> &ExecutorBuffer {
    self
  }
}

impl LeaseMut<ExecutorBuffer> for ExecutorBuffer {
  #[inline]
  fn lease_mut(&mut self) -> &mut ExecutorBuffer {
    self
  }
}

pub(crate) struct ExecutorBufferPartsMut<'eb> {
  pub(crate) nb: &'eb mut PartitionedFilledBuffer,
  pub(crate) params: &'eb mut Vec<(Identifier, Identifier)>,
  pub(crate) rb: &'eb mut Vec<usize>,
  pub(crate) stmts: &'eb mut Statements,
  pub(crate) vb: &'eb mut Vec<(bool, Range<usize>)>,
}
