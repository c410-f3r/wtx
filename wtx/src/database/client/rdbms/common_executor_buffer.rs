use crate::{
  collection::Vector,
  database::client::rdbms::statements::Statements,
  misc::{Lease, LeaseMut, net::PartitionedFilledBuffer},
  rng::Rng,
};
use core::ops::Range;

#[derive(Debug)]
pub(crate) struct CommonExecutorBuffer<A, C, T> {
  pub(crate) net_buffer: PartitionedFilledBuffer,
  pub(crate) records_params: Vector<(Range<usize>, Range<usize>)>,
  pub(crate) stmts: Statements<A, C, T>,
  pub(crate) values_params: Vector<(bool, Range<usize>)>,
}

impl<A, C, T> CommonExecutorBuffer<A, C, T> {
  /// With provided capacity.
  #[inline]
  pub(crate) fn new<RNG>(max_stmts: usize, rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      net_buffer: PartitionedFilledBuffer::new(),
      records_params: Vector::new(),
      stmts: Statements::new(max_stmts, rng),
      values_params: Vector::new(),
    }
  }

  /// With default capacity.
  #[inline]
  pub(crate) fn with_capacity<RNG>(
    (columns_cap, network_buffer_cap, rows_cap, stmts_cap): (usize, usize, usize, usize),
    max_stmts: usize,
    rng: &mut RNG,
  ) -> crate::Result<Self>
  where
    RNG: Rng,
  {
    Ok(Self {
      net_buffer: PartitionedFilledBuffer::_with_capacity(network_buffer_cap)?,
      records_params: Vector::with_capacity(rows_cap)?,
      stmts: Statements::with_capacity(columns_cap, max_stmts, rng, stmts_cap)?,
      values_params: Vector::with_capacity(rows_cap.saturating_mul(columns_cap))?,
    })
  }

  /// Should be used in a new instance.
  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { net_buffer, records_params, stmts, values_params } = self;
    net_buffer._clear();
    records_params.clear();
    stmts.clear();
    values_params.clear();
  }
}

impl<A, C, T> Lease<CommonExecutorBuffer<A, C, T>> for CommonExecutorBuffer<A, C, T> {
  #[inline]
  fn lease(&self) -> &CommonExecutorBuffer<A, C, T> {
    self
  }
}

impl<A, C, T> LeaseMut<CommonExecutorBuffer<A, C, T>> for CommonExecutorBuffer<A, C, T> {
  #[inline]
  fn lease_mut(&mut self) -> &mut CommonExecutorBuffer<A, C, T> {
    self
  }
}
