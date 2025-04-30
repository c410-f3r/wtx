use crate::{
  database::{Identifier, client::postgres::PostgresCommonExecutorBuffer},
  misc::{Lease, LeaseMut},
  rng::Rng,
};
use hashbrown::HashMap;

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct ExecutorBuffer {
  pub(crate) common: PostgresCommonExecutorBuffer,
  pub(crate) conn_params: HashMap<Identifier, Identifier>,
}

impl ExecutorBuffer {
  /// New instance
  #[inline]
  pub fn new<RNG>(max_stmts: usize, rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self { common: PostgresCommonExecutorBuffer::new(max_stmts, rng), conn_params: HashMap::new() }
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
      common: PostgresCommonExecutorBuffer::with_capacity(
        (columns_cap, network_buffer_cap, rows_cap, stmts_cap),
        max_stmts,
        rng,
      )?,
      conn_params: HashMap::with_capacity(4),
    })
  }

  /// Should be used in a new instance.
  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { common, conn_params } = self;
    common.clear();
    conn_params.clear();
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
