use crate::{
  database::{client::mysql::Mysql, ValueIdent},
  misc::DEController,
};
use core::marker::PhantomData;

/// Record
#[derive(Debug)]
pub struct Record<'exec, E> {
  pub(crate) bytes: &'exec [u8],
  pub(crate) phantom: PhantomData<fn() -> E>,
}

impl<'exec, E> crate::database::Record<'exec> for Record<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Mysql<E>;

  #[inline]
  fn len(&self) -> usize {
    0
  }

  #[inline]
  fn value<CI>(&self, _: CI) -> Option<<Self::Database as DEController>::DecodeWrapper<'_, 'exec>>
  where
    CI: ValueIdent<Self>,
  {
    None
  }
}

impl<'exec, E> ValueIdent<Record<'exec, E>> for str {
  #[inline]
  fn idx(&self, _: &Record<'exec, E>) -> Option<usize> {
    None
  }
}

mod array {
  use crate::{
    database::{client::mysql::Mysql, FromRecord, Record},
    misc::{from_utf8_basic, into_rslt, ArrayString},
  };

  impl<E, const N: usize> FromRecord<Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn from_record(record: &crate::database::client::mysql::Record<'_, E>) -> Result<Self, E> {
      Ok(
        from_utf8_basic(into_rslt(record.value(0))?.bytes())
          .map_err(From::from)?
          .try_into()
          .map_err(From::from)?,
      )
    }
  }
}
