use crate::{
  collections::Vector,
  database::client::rdbms::statements::Statements,
  misc::{Lease, LeaseMut},
  rng::Rng,
  stream::BufStreamReader,
};
use core::ops::Range;

#[derive(Debug)]
pub(crate) struct CommonClientBuffer<A, C, T> {
  pub(crate) read_buffer: BufStreamReader,
  /// Each element represents a ***whole*** record. The first element is the number of affected
  /// values, the second element is the range delimitates bytes and the third element if the range
  /// that delimitates `values_params`.
  pub(crate) records_params: Vector<(Range<usize>, Range<usize>)>,
  pub(crate) stmts: Statements<A, C, T>,
  /// Each element represents the ***data*** of a record that is delimited by the first range of
  /// `records_params`.
  pub(crate) values_params: Vector<(bool, Range<usize>)>,
}

impl<A, C, T> CommonClientBuffer<A, C, T> {
  /// With provided capacity.
  pub(crate) fn new<RNG>(max_stmts: usize, rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self {
      read_buffer: BufStreamReader::new(),
      records_params: Vector::new(),
      stmts: Statements::new(max_stmts, rng),
      values_params: Vector::new(),
    }
  }

  /// Should be used in a new instance.
  pub(crate) fn clear(&mut self) {
    let Self { read_buffer, records_params, stmts, values_params } = self;
    read_buffer.clear();
    records_params.clear();
    stmts.clear();
    values_params.clear();
  }
}

impl<A, C, T> Lease<CommonClientBuffer<A, C, T>> for CommonClientBuffer<A, C, T> {
  #[inline]
  fn lease(&self) -> &CommonClientBuffer<A, C, T> {
    self
  }
}

impl<A, C, T> LeaseMut<CommonClientBuffer<A, C, T>> for CommonClientBuffer<A, C, T> {
  #[inline]
  fn lease_mut(&mut self) -> &mut CommonClientBuffer<A, C, T> {
    self
  }
}
