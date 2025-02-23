use crate::{
  database::{Identifier, client::postgres::PostgresStatements},
  misc::{Lease, LeaseMut, Rng, Vector, partitioned_filled_buffer::PartitionedFilledBuffer},
};
use core::ops::Range;
use hashbrown::HashMap;

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct ExecutorBuffer {
  /// Connection parameters.
  pub(crate) cp: HashMap<Identifier, Identifier>,
  /// Network Buffer.
  pub(crate) nb: PartitionedFilledBuffer,
  /// Records Buffer.
  pub(crate) rb: Vector<usize>,
  /// Statements
  pub(crate) stmts: PostgresStatements,
  /// Values Buffer.
  pub(crate) vb: Vector<(bool, Range<usize>)>,
}

impl ExecutorBuffer {
  /// With provided capacity.
  #[inline]
  pub fn new<RNG>(max_stmts: usize, rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      cp: HashMap::new(),
      nb: PartitionedFilledBuffer::new(),
      rb: Vector::new(),
      stmts: PostgresStatements::new(max_stmts, rng),
      vb: Vector::new(),
    }
  }

  /// With default capacity.
  #[inline]
  pub fn with_capacity<RNG>(
    (columns_cap, network_buffer_cap, rows_cap, stmts_cap): (usize, usize, usize, usize),
    max_stmts: usize,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    Ok(Self {
      cp: HashMap::with_capacity(4),
      nb: PartitionedFilledBuffer::_with_capacity(network_buffer_cap)?,
      rb: Vector::with_capacity(rows_cap)?,
      stmts: PostgresStatements::with_capacity(columns_cap, max_stmts, rng, stmts_cap)?,
      vb: Vector::with_capacity(rows_cap.saturating_mul(columns_cap))?,
    })
  }

  /// Should be used in a new instance.
  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { cp, nb, rb, stmts, vb } = self;
    cp.clear();
    nb._clear();
    rb.clear();
    stmts.clear();
    vb.clear();
  }

  /// Should be called before executing commands.
  #[inline]
  pub(crate) fn clear_cmd_buffers(
    nb: &mut PartitionedFilledBuffer,
    rb: &mut Vector<usize>,
    vb: &mut Vector<(bool, Range<usize>)>,
  ) {
    nb._clear_if_following_is_empty();
    rb.clear();
    vb.clear();
  }

  #[inline]
  pub(crate) fn parts_mut(&mut self) -> ExecutorBufferPartsMut<'_> {
    ExecutorBufferPartsMut {
      cp: &mut self.cp,
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
  pub(crate) cp: &'eb mut HashMap<Identifier, Identifier>,
  pub(crate) nb: &'eb mut PartitionedFilledBuffer,
  pub(crate) rb: &'eb mut Vector<usize>,
  pub(crate) stmts: &'eb mut PostgresStatements,
  pub(crate) vb: &'eb mut Vector<(bool, Range<usize>)>,
}
