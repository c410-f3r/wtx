use crate::{
  database::{client::postgres::Statements, Identifier},
  misc::PartitionedFilledBuffer,
  rng::Rng,
};
use alloc::vec::Vec;
use core::ops::Range;

pub(crate) const DFLT_PARAMS_LEN: usize = 16;
pub(crate) const DFLT_RECORDS_LEN: usize = 32;
pub(crate) const DFLT_VALUES_LEN: usize = 16;

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct ExecutorBuffer {
  /// Network Buffer.
  pub(crate) nb: PartitionedFilledBuffer,
  /// Asynchronous parameters received from the database.
  pub(crate) params: Vec<(Identifier, Identifier)>,
  /// Records Buffer.
  pub(crate) rb: Vec<usize>,
  /// Statements
  pub(crate) stmts: Statements,
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
      nb: PartitionedFilledBuffer::with_capacity(network_buffer_cap),
      params: Vec::with_capacity(DFLT_PARAMS_LEN),
      rb: Vec::with_capacity(records_buffer_cap),
      stmts: Statements::new(max_queries, rng),
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
      nb: PartitionedFilledBuffer::default(),
      params: Vec::with_capacity(DFLT_PARAMS_LEN),
      rb: Vec::with_capacity(DFLT_RECORDS_LEN),
      stmts: Statements::with_default_params(rng),
      vb: Vec::with_capacity(DFLT_VALUES_LEN),
    }
  }

  pub(crate) fn _empty() -> Self {
    Self {
      nb: PartitionedFilledBuffer::_empty(),
      params: Vec::new(),
      rb: Vec::new(),
      stmts: Statements::_empty(),
      vb: Vec::new(),
    }
  }

  /// Should be used in running instances.
  pub(crate) fn clear(&mut self) {
    let Self { nb, params: _, rb, stmts: _, vb } = self;
    nb._clear_if_following_is_empty();
    rb.clear();
    vb.clear();
  }

  /// Should be used in a new instance.
  pub(crate) fn clear_all(&mut self) {
    let Self { nb, params, rb, stmts, vb } = self;
    nb._clear();
    params.clear();
    rb.clear();
    stmts.clear();
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

pub(crate) struct ExecutorBufferPartsMut<'eb> {
  pub(crate) nb: &'eb mut PartitionedFilledBuffer,
  pub(crate) params: &'eb mut Vec<(Identifier, Identifier)>,
  pub(crate) rb: &'eb mut Vec<usize>,
  pub(crate) stmts: &'eb mut Statements,
  pub(crate) vb: &'eb mut Vec<(bool, Range<usize>)>,
}
