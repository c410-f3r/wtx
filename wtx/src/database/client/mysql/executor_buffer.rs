use crate::{
  collection::{Clear, Vector},
  database::client::mysql::MysqlCommonExecutorBuffer,
  misc::{Lease, LeaseMut},
  rng::Rng,
};

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct ExecutorBuffer {
  pub(crate) common: MysqlCommonExecutorBuffer,
  pub(crate) encode_buffer: Vector<u8>,
}

impl ExecutorBuffer {
  /// New instance
  #[inline]
  pub fn new<RNG>(max_stmts: usize, rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self { common: MysqlCommonExecutorBuffer::new(max_stmts, rng), encode_buffer: Vector::new() }
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
      common: MysqlCommonExecutorBuffer::with_capacity(
        (columns_cap, network_buffer_cap, rows_cap, stmts_cap),
        max_stmts,
        rng,
      )?,
      encode_buffer: Vector::with_capacity(enc_cap)?,
    })
  }

  /// Removes inner content
  #[inline]
  pub fn clear(&mut self) {
    let Self { common, encode_buffer } = self;
    common.clear();
    encode_buffer.clear();
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
