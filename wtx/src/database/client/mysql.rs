//! MySQL is an open source relational database management system (RDBMS) that's used to store
//! and manage data.

mod auth_plugin;
mod capability;
pub(crate) mod charset;
pub(crate) mod collation;
mod column;
mod config;
mod db_error;
mod decode_wrapper;
mod encode_wrapper;
mod executor_buffer;
mod flag;
#[cfg(all(feature = "_async-tests", feature = "_integration-tests", test))]
mod integration_tests;
mod misc;
mod mysql_error;
mod mysql_executor;
mod mysql_protocol;
mod mysql_record;
mod mysql_records;
mod status;
mod ty;
mod ty_params;
mod tys;

use crate::{
  database::{
    Database, DatabaseTy,
    client::rdbms::{
      common_executor_buffer::CommonExecutorBuffer,
      common_record::CommonRecord,
      common_records::CommonRecords,
      statement::{Statement, StatementMut},
      statements::Statements,
    },
  },
  misc::DEController,
};
pub use config::Config;
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
};
pub use db_error::DbError;
pub use decode_wrapper::DecodeWrapper;
pub use encode_wrapper::EncodeWrapper;
pub use executor_buffer::ExecutorBuffer;
pub use mysql_error::MysqlError;
pub use mysql_executor::MysqlExecutor;
pub use mysql_record::MysqlRecord;
pub use mysql_records::MysqlRecords;
pub use ty::Ty;
pub use ty_params::TyParams;

pub(crate) type MysqlCommonRecord<'exec, E> =
  CommonRecord<'exec, u32, column::Column, Mysql<E>, TyParams>;
pub(crate) type MysqlCommonRecords<'exec, E> =
  CommonRecords<'exec, u32, column::Column, Mysql<E>, TyParams>;
pub(crate) type MysqlCommonExecutorBuffer = CommonExecutorBuffer<u32, column::Column, TyParams>;
pub(crate) type MysqlStatements = Statements<u32, column::Column, TyParams>;
pub(crate) type MysqlStatement<'stmts> = Statement<'stmts, u32, column::Column, TyParams>;
pub(crate) type MysqlStatementMut<'stmts> = StatementMut<'stmts, u32, column::Column, TyParams>;

/// MySQL
pub struct Mysql<E>(PhantomData<fn() -> E>);

impl<E> Database for Mysql<E>
where
  E: From<crate::Error>,
{
  const TY: DatabaseTy = DatabaseTy::Mysql;

  type Record<'exec> = MysqlRecord<'exec, E>;
  type Records<'exec> = MysqlRecords<'exec, E>;
  type Ty = TyParams;
}

impl<E> DEController for Mysql<E>
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
    = EncodeWrapper<'inner>
  where
    'inner: 'outer;
}

impl<E> Debug for Mysql<E> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Mysql").finish()
  }
}

impl<E> Default for Mysql<E> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
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
