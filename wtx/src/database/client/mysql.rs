//! MySQL is an open source relational database management system (RDBMS) that's used to store
//! and manage data.

mod auth_plugin;
mod capability;
pub(crate) mod charset;
pub(crate) mod collation;
mod config;
mod db_error;
mod executor_buffer;
mod flag;
#[cfg(all(feature = "_integration-tests", test))]
mod integration_tests;
mod misc;
mod mysql_column_info;
mod mysql_decode_wrapper;
mod mysql_encode_wrapper;
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
  de::DEController,
};
pub use config::Config;
use core::{
  fmt::{Debug, Formatter},
  marker::PhantomData,
};
pub use db_error::DbError;
pub use executor_buffer::ExecutorBuffer;
pub use mysql_decode_wrapper::MysqlDecodeWrapper;
pub use mysql_encode_wrapper::MysqlEncodeWrapper;
pub use mysql_error::MysqlError;
pub use mysql_executor::MysqlExecutor;
pub use mysql_record::MysqlRecord;
pub use mysql_records::MysqlRecords;
pub use ty::Ty;
pub use ty_params::TyParams;

pub(crate) type MysqlCommonRecord<'exec, E> =
  CommonRecord<'exec, u32, mysql_column_info::MysqlColumnInfo, Mysql<E>, TyParams>;
pub(crate) type MysqlCommonRecords<'exec, E> =
  CommonRecords<'exec, u32, mysql_column_info::MysqlColumnInfo, Mysql<E>, TyParams>;
pub(crate) type MysqlCommonExecutorBuffer =
  CommonExecutorBuffer<u32, mysql_column_info::MysqlColumnInfo, TyParams>;
pub(crate) type MysqlStatements = Statements<u32, mysql_column_info::MysqlColumnInfo, TyParams>;
pub(crate) type MysqlStatement<'stmts> =
  Statement<'stmts, u32, mysql_column_info::MysqlColumnInfo, TyParams>;
pub(crate) type MysqlStatementMut<'stmts> =
  StatementMut<'stmts, u32, mysql_column_info::MysqlColumnInfo, TyParams>;

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
  type DecodeWrapper<'inner, 'outer, 'rem>
    = MysqlDecodeWrapper<'inner, 'rem>
  where
    'inner: 'outer;
  type Error = E;
  type EncodeWrapper<'inner, 'outer, 'rem>
    = MysqlEncodeWrapper<'inner>
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
    collection::{ArrayString, LinearStorageLen},
    database::{
      FromRecords, FromRecordsParams, Record,
      client::mysql::{Mysql, MysqlRecord, MysqlRecords},
    },
    misc::{from_utf8_basic, into_rslt},
  };

  impl<'exec, E, L, const N: usize> FromRecords<'exec, Mysql<E>> for ArrayString<L, N>
  where
    E: From<crate::Error>,
    L: LinearStorageLen,
  {
    const FIELDS: u16 = 1;
    const ID_IDX: Option<usize> = None;
    type IdTy = ();

    #[inline]
    fn from_records(
      curr_params: &mut FromRecordsParams<MysqlRecord<'exec, E>>,
      _: &MysqlRecords<'_, E>,
    ) -> Result<Self, E> {
      let rslt = from_utf8_basic(into_rslt(curr_params.curr_record.value(0))?.bytes())
        .map_err(From::from)?
        .try_into()?;
      curr_params.inc_consumed_records(1);
      Ok(rslt)
    }
  }
}
