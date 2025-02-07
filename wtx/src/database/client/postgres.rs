//! Postgres, is a free and open-source relational database management system (RDBMS) emphasizing
//! extensibility and SQL compliance.

mod authentication;
mod config;
mod db_error;
mod decode_wrapper;
mod encode_wrapper;
mod executor;
mod executor_buffer;
#[cfg(all(feature = "_async-tests", feature = "_integration-tests", test))]
mod integration_tests;
mod message;
mod msg_field;
mod postgres_error;
mod protocol;
mod record;
mod records;
mod sql_state;
mod statements;
mod struct_decoder;
mod struct_encoder;
mod ty;
mod tys;

use crate::{
  database::{Database, DatabaseTy},
  misc::DEController,
};
pub use config::Config;
use core::marker::PhantomData;
pub use db_error::{DbError, ErrorPosition, Severity};
pub use decode_wrapper::DecodeWrapper;
pub use encode_wrapper::EncodeWrapper;
pub use executor::PostgresExecutor;
pub use executor_buffer::ExecutorBuffer;
pub use postgres_error::PostgresError;
pub use record::PostgresRecord;
pub use records::PostgresRecords;
pub use sql_state::SqlState;
pub use statements::Statements;
pub use struct_decoder::StructDecoder;
pub use struct_encoder::StructEncoder;
pub use ty::Ty;

pub(crate) type Oid = u32;

/// Postgres
#[derive(Debug)]
pub struct Postgres<E>(PhantomData<fn() -> E>);

impl<E> Database for Postgres<E>
where
  E: From<crate::Error>,
{
  const TY: DatabaseTy = DatabaseTy::Postgres;

  type Record<'exec> = PostgresRecord<'exec, E>;
  type Records<'exec> = PostgresRecords<'exec, E>;
  type Ty = Ty;
}

impl<E> DEController for Postgres<E>
where
  E: From<crate::Error>,
{
  type Aux = ();
  type DecodeWrapper<'de> = DecodeWrapper<'de>;
  type Error = E;
  type EncodeWrapper<'inner, 'outer>
    = EncodeWrapper<'inner, 'outer>
  where
    'inner: 'outer;
}

impl<E> Default for Postgres<E> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
  }
}

#[cfg(test)]
mod tests {
  use crate::database::client::postgres::{Ty, statements::column::Column};

  pub(crate) fn column0() -> Column {
    Column::new("a".try_into().unwrap(), Ty::VarcharArray)
  }

  pub(crate) fn column1() -> Column {
    Column::new("b".try_into().unwrap(), Ty::Int8)
  }

  pub(crate) fn column2() -> Column {
    Column::new("c".try_into().unwrap(), Ty::Char)
  }

  pub(crate) fn column3() -> Column {
    Column::new("d".try_into().unwrap(), Ty::Date)
  }
}
