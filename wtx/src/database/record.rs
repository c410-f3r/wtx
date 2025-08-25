use crate::{
  database::{Database, DatabaseError, ValueIdent},
  de::{DEController, Decode},
};
use core::any::type_name;

/// A collection of values.
pub trait Record<'exec>: Sized {
  /// See [Database].
  type Database: Database<Aux = ()>;

  /// Tries to retrieve and decode a value.
  #[inline]
  fn decode<CI, D>(&self, ci: CI) -> Result<D, <Self::Database as DEController>::Error>
  where
    CI: ValueIdent<Self>,
    D: Decode<'exec, Self::Database>,
  {
    let mut dw = self.value(ci).ok_or_else(|| {
      DatabaseError::MissingFieldDataInDecoding(
        alloc::format!("{:?} - {}", ci.idx(self), type_name::<D>()).into(),
      )
      .into()
    })?;
    D::decode(&mut (), &mut dw)
  }

  /// Tries to retrieve and decode an optional value.
  #[inline]
  fn decode_opt<CI, D>(&self, ci: CI) -> Result<Option<D>, <Self::Database as DEController>::Error>
  where
    CI: ValueIdent<Self>,
    D: Decode<'exec, Self::Database>,
  {
    match self.value(ci) {
      Some(mut elem) => Ok(Some(D::decode(&mut (), &mut elem)?)),
      None => Ok(None),
    }
  }

  /// The number of values.
  fn len(&self) -> usize;

  /// Tries to retrieve a value.
  fn value<CI>(&self, ci: CI) -> Option<<Self::Database as DEController>::DecodeWrapper<'exec, '_>>
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
  fn value<CI>(&self, _: CI) -> Option<<Self::Database as DEController>::DecodeWrapper<'_, '_>>
  where
    CI: ValueIdent<Self>,
  {
    None
  }
}
