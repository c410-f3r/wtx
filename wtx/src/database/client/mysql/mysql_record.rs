use crate::{
  database::{ValueIdent, client::mysql::Mysql},
  misc::DEController,
};
use core::marker::PhantomData;

/// Record
#[derive(Debug)]
pub struct MysqlRecord<'exec, E> {
  pub(crate) bytes: &'exec [u8],
  pub(crate) phantom: PhantomData<fn() -> E>,
}

impl<'exec, E> crate::database::Record<'exec> for MysqlRecord<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Mysql<E>;

  #[inline]
  fn len(&self) -> usize {
    0
  }

  #[inline]
  fn value<CI>(&self, _: CI) -> Option<<Self::Database as DEController>::DecodeWrapper<'exec>>
  where
    CI: ValueIdent<Self>,
  {
    None
  }
}

impl<'exec, E> ValueIdent<MysqlRecord<'exec, E>> for str {
  #[inline]
  fn idx(&self, _: &MysqlRecord<'exec, E>) -> Option<usize> {
    None
  }
}

mod array {
  use crate::{
    database::{FromRecord, Record, client::mysql::Mysql},
    misc::{ArrayString, from_utf8_basic, into_rslt},
  };

  impl<E, const N: usize> FromRecord<Mysql<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn from_record(record: &crate::database::client::mysql::MysqlRecord<'_, E>) -> Result<Self, E> {
      Ok(from_utf8_basic(into_rslt(record.value(0))?.bytes()).map_err(From::from)?.try_into()?)
    }
  }
}
