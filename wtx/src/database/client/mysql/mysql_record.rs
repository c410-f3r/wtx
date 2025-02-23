use crate::{
  database::{
    Record, ValueIdent,
    client::mysql::{DecodeWrapper, Mysql, MysqlStatement},
  },
  misc::{_unlikely_elem, DEController},
};
use core::{marker::PhantomData, ops::Range};

/// Record
#[derive(Debug)]
pub struct MysqlRecord<'any, E> {
  pub(crate) bytes: &'any [u8],
  pub(crate) phantom: PhantomData<fn() -> E>,
  pub(crate) stmt: MysqlStatement<'any>,
  pub(crate) values_bytes_offsets: &'any [(bool, Range<usize>)],
}

impl<'any, E> MysqlRecord<'any, E> {
  #[inline]
  pub(crate) fn new(
    bytes: &'any [u8],
    stmt: MysqlStatement<'any>,
    values_bytes_offsets: &'any [(bool, Range<usize>)],
  ) -> Self {
    Self { bytes, phantom: PhantomData, stmt, values_bytes_offsets }
  }
}

impl<'exec, E> Record<'exec> for MysqlRecord<'exec, E>
where
  E: From<crate::Error>,
{
  type Database = Mysql<E>;

  #[inline]
  fn len(&self) -> usize {
    0
  }

  #[inline]
  fn value<CI>(&self, ci: CI) -> Option<<Self::Database as DEController>::DecodeWrapper<'exec, '_>>
  where
    CI: ValueIdent<Self>,
  {
    let idx = ci.idx(self)?;
    let (is_null, range) = match self.values_bytes_offsets.get(idx) {
      None => return _unlikely_elem(None),
      Some(elem) => elem,
    };
    if *is_null {
      None
    } else {
      let column = match self.stmt._column(idx) {
        None => return _unlikely_elem(None),
        Some(elem) => elem,
      };
      let bytes = match self.bytes.get(range.clone()) {
        None => return _unlikely_elem(None),
        Some(elem) => elem,
      };
      Some(DecodeWrapper::new(bytes, column.ty_params.ty))
    }
  }
}

impl<'exec, E> ValueIdent<MysqlRecord<'exec, E>> for str {
  #[inline]
  fn idx(&self, input: &MysqlRecord<'exec, E>) -> Option<usize> {
    input.stmt._columns().position(|column| column.name.as_str() == self)
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
