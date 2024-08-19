use crate::{
  database::{
    client::postgres::{ty::Ty, Statements},
    Identifier,
  },
  misc::{Lease, LeaseMut, PartitionedFilledBuffer, Rng, Vector},
};
use core::ops::Range;
use hashbrown::HashMap;

pub(crate) const DFLT_PARAMS_LEN: usize = 16;
pub(crate) const DFLT_RECORDS_LEN: usize = 32;
pub(crate) const DFLT_VALUES_LEN: usize = 16;

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct ExecutorBuffer {
  /// Asynchronous parameters received from the database.
  pub(crate) conn_params: Vector<(Identifier, Identifier)>,
  /// Fetch type buffer
  pub(crate) ftb: Vector<(usize, u32)>,
  /// Network Buffer.
  pub(crate) nb: PartitionedFilledBuffer,
  /// Records Buffer.
  pub(crate) rb: Vector<usize>,
  /// Statements
  pub(crate) stmts: Statements,
  /// Types buffer
  pub(crate) tb: HashMap<u32, Ty>,
  /// Values Buffer.
  pub(crate) vb: Vector<(bool, Range<usize>)>,
}

impl ExecutorBuffer {
  /// With provided capacity.
  #[inline]
  pub fn new<RNG>(
    (network_buffer_cap, records_buffer_cap, values_buffer_cap): (usize, usize, usize),
    rng: &mut RNG,
    max_queries: usize,
  ) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    Ok(Self {
      conn_params: Vector::with_capacity(DFLT_PARAMS_LEN)?,
      ftb: Vector::new(),
      nb: PartitionedFilledBuffer::_with_capacity(network_buffer_cap),
      rb: Vector::with_capacity(records_buffer_cap)?,
      stmts: Statements::new(max_queries, rng),
      tb: HashMap::new(),
      vb: Vector::with_capacity(values_buffer_cap)?,
    })
  }

  /// With default capacity.
  #[inline]
  pub fn with_default_params<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    Ok(Self {
      conn_params: Vector::with_capacity(DFLT_PARAMS_LEN)?,
      ftb: Vector::new(),
      nb: PartitionedFilledBuffer::default(),
      rb: Vector::with_capacity(DFLT_RECORDS_LEN)?,
      stmts: Statements::with_default_params(rng),
      tb: HashMap::new(),
      vb: Vector::with_capacity(DFLT_VALUES_LEN)?,
    })
  }

  pub(crate) fn _empty() -> Self {
    Self {
      conn_params: Vector::new(),
      ftb: Vector::new(),
      nb: PartitionedFilledBuffer::new(),
      rb: Vector::new(),
      stmts: Statements::_empty(),
      tb: HashMap::new(),
      vb: Vector::new(),
    }
  }

  /// Should be used in a new instance.
  pub(crate) fn clear(&mut self) {
    let Self { conn_params, ftb, nb, rb, stmts, tb, vb } = self;
    conn_params.clear();
    ftb.clear();
    nb._clear();
    rb.clear();
    stmts.clear();
    tb.clear();
    vb.clear();
  }

  /// Should be called before executing commands.
  pub(crate) fn clear_cmd_buffers(
    nb: &mut PartitionedFilledBuffer,
    rb: &mut Vector<usize>,
    vb: &mut Vector<(bool, Range<usize>)>,
  ) {
    nb._clear_if_following_is_empty();
    rb.clear();
    vb.clear();
  }

  pub(crate) fn parts_mut(&mut self) -> ExecutorBufferPartsMut<'_> {
    ExecutorBufferPartsMut {
      conn_params: &mut self.conn_params,
      nb: &mut self.nb,
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
  pub(crate) conn_params: &'eb mut Vector<(Identifier, Identifier)>,
  pub(crate) nb: &'eb mut PartitionedFilledBuffer,
  pub(crate) rb: &'eb mut Vector<usize>,
  pub(crate) stmts: &'eb mut Statements,
  pub(crate) vb: &'eb mut Vector<(bool, Range<usize>)>,
}
