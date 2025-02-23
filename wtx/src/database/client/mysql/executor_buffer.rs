use crate::{
  database::client::mysql::MysqlStatements,
  misc::{Lease, LeaseMut, Rng, Vector, partitioned_filled_buffer::PartitionedFilledBuffer},
};
use core::ops::Range;

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct ExecutorBuffer {
  pub(crate) enc_buffer: Vector<u8>,
  pub(crate) net_buffer: PartitionedFilledBuffer,
  pub(crate) stmts: MysqlStatements,
  pub(crate) vb: Vector<(bool, Range<usize>)>,
}

impl ExecutorBuffer {
  /// New instance
  #[inline]
  pub fn new<RNG>(max_stmts: usize, rng: RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      enc_buffer: Vector::new(),
      net_buffer: PartitionedFilledBuffer::new(),
      stmts: MysqlStatements::new(max_stmts, rng),
      vb: Vector::new(),
    }
  }

  /// With default capacity.
  #[inline]
  pub fn with_capacity<RNG>(
    (columns_cap, enc_cap, network_buffer_cap, rows_cap, stmts_cap): (
      usize,
      usize,
      usize,
      usize,
      usize,
    ),
    max_stmts: usize,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    Ok(Self {
      enc_buffer: Vector::with_capacity(enc_cap)?,
      net_buffer: PartitionedFilledBuffer::_with_capacity(network_buffer_cap)?,
      stmts: MysqlStatements::with_capacity(columns_cap, max_stmts, rng, stmts_cap)?,
      vb: Vector::with_capacity(rows_cap.saturating_mul(columns_cap))?,
    })
  }

  /// Removes inner content
  #[inline]
  pub fn clear(&mut self) {
    let Self { enc_buffer, net_buffer, stmts, vb } = self;
    enc_buffer.clear();
    net_buffer._clear();
    stmts.clear();
    vb.clear();
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
