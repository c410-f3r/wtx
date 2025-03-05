//! Postgres, is a free and open-source relational database management system (RDBMS) emphasizing
//! extensibility and SQL compliance.

mod authentication;
mod column;
mod config;
mod db_error;
mod decode_wrapper;
mod encode_wrapper;
mod executor_buffer;
#[cfg(all(feature = "_async-tests", feature = "_integration-tests", test))]
mod integration_tests;
mod message;
mod msg_field;
mod postgres_error;
mod postgres_executor;
mod postgres_record;
mod postgres_records;
mod protocol;
mod sql_state;
mod struct_decoder;
mod struct_encoder;
mod ty;
mod tys;

use crate::{
  database::{
    Database, DatabaseTy,
    client::rdbms::{
      common_executor_buffer::CommonExecutorBuffer, common_record::CommonRecord,
      common_records::CommonRecords, statement::Statement, statements::Statements,
    },
  },
  misc::{DEController, U64String},
};
pub use config::Config;
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
};
pub use db_error::{DbError, ErrorPosition, Severity};
pub use decode_wrapper::DecodeWrapper;
pub use encode_wrapper::EncodeWrapper;
pub use executor_buffer::ExecutorBuffer;
pub use postgres_error::PostgresError;
pub use postgres_executor::PostgresExecutor;
pub use postgres_record::PostgresRecord;
pub use postgres_records::PostgresRecords;
pub use sql_state::SqlState;
pub use struct_decoder::StructDecoder;
pub use struct_encoder::StructEncoder;
pub use ty::Ty;

pub(crate) type Oid = u32;
pub(crate) type PostgresCommonRecord<'exec, E> =
  CommonRecord<'exec, U64String, column::Column, Postgres<E>, Ty>;
pub(crate) type PostgresCommonRecords<'exec, E> =
  CommonRecords<'exec, U64String, column::Column, Postgres<E>, Ty>;
pub(crate) type PostgresStatements = Statements<U64String, column::Column, Ty>;
pub(crate) type PostgresStatement<'stmts> = Statement<'stmts, U64String, column::Column, Ty>;
pub(crate) type PostgresCommonExecutorBuffer = CommonExecutorBuffer<U64String, column::Column, Ty>;

/// Postgres
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
  type DecodeWrapper<'inner, 'outer>
    = DecodeWrapper<'inner>
  where
    'inner: 'outer;
  type Error = E;
  type EncodeWrapper<'inner, 'outer>
    = EncodeWrapper<'inner, 'outer>
  where
    'inner: 'outer;
}

impl<E> Debug for Postgres<E> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Postgres").finish()
  }
}

impl<E> Default for Postgres<E> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
  }
}

mod array {
  use crate::{
    database::{
      FromRecord, Record,
      client::postgres::{Postgres, postgres_record::PostgresRecord},
    },
    misc::{ArrayString, from_utf8_basic, into_rslt},
  };

  impl<E, const N: usize> FromRecord<Postgres<E>> for ArrayString<N>
  where
    E: From<crate::Error>,
  {
    #[inline]
    fn from_record(record: &PostgresRecord<'_, E>) -> Result<Self, E> {
      Ok(from_utf8_basic(into_rslt(record.value(0))?.bytes()).map_err(From::from)?.try_into()?)
    }
  }
}
