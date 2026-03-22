use crate::{
  codec::CodecController,
  collection::Vector,
  database::schema_manager::{Commands, SchemaManagement},
};
use alloc::string::String;

impl<E> Commands<E>
where
  E: SchemaManagement,
{
  /// Tries to clear all objects of a database, including separated namespaces/schemas.
  #[inline]
  pub async fn clear(&mut self) -> Result<(), <E::Database as CodecController>::Error> {
    self.executor.clear((&mut String::new(), &mut Vector::new())).await
  }
}
