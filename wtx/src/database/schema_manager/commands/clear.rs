use crate::{
  collection::Vector,
  database::{
    Identifier,
    schema_manager::{Commands, SchemaManagement},
  },
};
use alloc::string::String;

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Tries to clear all objects of a database, including separated namespaces/schemas.
  #[inline]
  pub async fn clear(
    &mut self,
    buffer: (&mut String, &mut Vector<Identifier>),
  ) -> crate::Result<()> {
    self.executor.clear(buffer).await
  }
}
