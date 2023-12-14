use crate::database::{
  sm::{Commands, SchemaManagement},
  Identifier,
};
use alloc::{string::String, vec::Vec};

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Tries to clear all objects of a database, including separated namespaces/schemas.
  #[inline]
  pub async fn clear(&mut self, buffer: (&mut String, &mut Vec<Identifier>)) -> crate::Result<()> {
    self.executor.clear(buffer).await
  }
}
