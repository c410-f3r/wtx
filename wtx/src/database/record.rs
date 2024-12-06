use crate::database::{Database, DatabaseError, Decode, ValueIdent};

/// A collection of values.
pub trait Record<'exec>: Sized {
  /// See [Database].
  type Database: Database;

  /// Tries to retrieve and decode a value.
  #[inline]
  fn decode<CI, D>(&self, ci: CI) -> Result<D, <Self::Database as Database>::Error>
  where
    CI: ValueIdent<Self>,
    D: Decode<'exec, Self::Database>,
  {
    D::decode(&self.value(ci).ok_or_else(|| DatabaseError::MissingFieldDataInDecoding.into())?)
  }

  /// Tries to retrieve and decode an optional value.
  #[inline]
  fn decode_opt<CI, D>(&self, ci: CI) -> Result<Option<D>, <Self::Database as Database>::Error>
  where
    CI: ValueIdent<Self>,
    D: Decode<'exec, Self::Database>,
  {
    match self.value(ci) {
      Some(elem) => Ok(Some(D::decode(&elem)?)),
      None => Ok(None),
    }
  }

  /// The number of values.
  fn len(&self) -> usize;

  /// Tries to retrieve a value.
  fn value<CI>(&self, ci: CI) -> Option<<Self::Database as Database>::DecodeValue<'exec>>
  where
    CI: ValueIdent<Self>;
}

impl Record<'_> for () {
  type Database = ();

  #[inline]
  fn len(&self) -> usize {
    0
  }

  #[inline]
  fn value<CI>(&self, _: CI) -> Option<<Self::Database as Database>::DecodeValue<'_>>
  where
    CI: ValueIdent<Self>,
  {
    None
  }
}
