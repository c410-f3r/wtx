use crate::database::{Database, Decode, ValueIdent};

/// A collection of values.
pub trait Record {
  /// See [Database].
  type Database: Database;

  /// Tries to retrieve and decode a value.
  #[inline]
  fn decode<'this, CI, D>(&'this self, ci: CI) -> crate::Result<D>
  where
    CI: ValueIdent<<Self::Database as Database>::Record<'this>>,
    D: Decode<
      Self::Database,
      crate::Error,
      Value<'this> = <Self::Database as Database>::Value<'this>,
    >,
  {
    D::decode(self.value(ci).unwrap_or_default())
  }

  /// The number of values.
  fn len(&self) -> usize;

  /// Tries to retrieve a value.
  fn value<'this, CI>(&'this self, ci: CI) -> Option<<Self::Database as Database>::Value<'this>>
  where
    CI: ValueIdent<<Self::Database as Database>::Record<'this>>;
}

impl Record for () {
  type Database = ();

  #[inline]
  fn len(&self) -> usize {
    0
  }

  #[inline]
  fn value<'this, CI>(&'this self, _: CI) -> Option<<Self::Database as Database>::Value<'this>>
  where
    CI: ValueIdent<<Self::Database as Database>::Record<'this>>,
  {
    None
  }
}
