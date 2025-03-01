//! MySQL is an open source relational database management system (RDBMS) that's used to store
//! and manage data.

mod auth_plugin;
mod capability;
pub(crate) mod charset;
pub(crate) mod collation;
mod column;
mod config;
mod decode_wrapper;
mod encode_wrapper;
mod executor_buffer;
mod flags;
//#[cfg(all(feature = "_async-tests", feature = "_integration-tests", test))]
//mod integration_tests;
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
    client::rdbms::{statement::Statement, statements::Statements},
  },
  misc::DEController,
};
pub use config::Config;
use core::marker::PhantomData;
pub use decode_wrapper::DecodeWrapper;
pub use encode_wrapper::EncodeWrapper;
pub use executor_buffer::ExecutorBuffer;
pub use mysql_error::MysqlError;
pub use mysql_executor::MysqlExecutor;
pub use mysql_record::MysqlRecord;
pub use mysql_records::MysqlRecords;
pub use ty::Ty;

pub(crate) type MysqlStatements = Statements<u32, column::Column, ty_params::TyParams>;
pub(crate) type MysqlStatement<'stmts> =
  Statement<'stmts, u32, column::Column, ty_params::TyParams>;

/// MySQL
#[derive(Debug)]
pub struct Mysql<E>(PhantomData<fn() -> E>);

impl<E> Database for Mysql<E>
where
  E: From<crate::Error>,
{
  const TY: DatabaseTy = DatabaseTy::Mysql;

  type Record<'exec> = MysqlRecord<'exec, E>;
  type Records<'exec> = MysqlRecords<'exec, E>;
  type Ty = Ty;
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

impl<E> Default for Mysql<E> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
  }
}
