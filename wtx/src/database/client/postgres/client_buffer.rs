use crate::{
  database::{Identifier, client::postgres::PostgresCommonExecutorBuffer},
  rng::Rng,
  stream::BufStreamReader,
};
use hashbrown::HashMap;

#[derive(Debug)]
#[doc = _internal_buffer_doc!()]
pub struct ClientBuffer {
  pub(crate) common: PostgresCommonExecutorBuffer,
  pub(crate) conn_params: HashMap<Identifier, Identifier>,
}

impl ClientBuffer {
  /// New instance
  #[inline]
  pub fn new<RNG>(max_stmts: usize, rng: &mut RNG) -> Self
  where
    RNG: Rng,
  {
    Self { common: PostgresCommonExecutorBuffer::new(max_stmts, rng), conn_params: HashMap::new() }
  }

  /// See [`BufStreamReader`].
  #[inline]
  pub const fn read_buffer_mut(&mut self) -> &mut BufStreamReader {
    &mut self.common.read_buffer
  }

  /// Should be used in a new instance.
  pub(crate) fn clear(&mut self) {
    let Self { common, conn_params } = self;
    common.clear();
    conn_params.clear();
  }
}
