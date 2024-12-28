#![allow(dead_code)]

pub(crate) mod charset;
pub(crate) mod collation;
mod config;
mod decode_wrapper;
mod encode_wrapper;
mod executor;
mod executor_buffer;
mod mysql_error;
mod record;
mod records;
mod transaction_manager;
mod ty;
mod tys;

use crate::{
  database::{Database, DatabaseTy},
  misc::DEController,
};
pub use config::Config;
use core::marker::PhantomData;
pub use decode_wrapper::DecodeWrapper;
pub use encode_wrapper::EncodeWrapper;
pub use executor::Executor;
pub use executor_buffer::ExecutorBuffer;
pub use mysql_error::MysqlError;
pub use record::Record;
pub use records::Records;
pub use transaction_manager::TransactionManager;
pub use ty::Ty;

/// MySQL
#[derive(Debug)]
pub struct Mysql<E>(PhantomData<fn() -> E>);

impl<E> Database for Mysql<E>
where
  E: From<crate::Error>,
{
  const TY: DatabaseTy = DatabaseTy::Mysql;

  type Record<'exec> = Record<'exec, E>;
  type Records<'exec> = Records<'exec, E>;
  type Ty = Ty;
}

impl<E> DEController for Mysql<E>
where
  E: From<crate::Error>,
{
  type DecodeWrapper<'any, 'de> = DecodeWrapper<'de>;
  type Error = E;
  type EncodeWrapper<'inner, 'outer>
    = EncodeWrapper<'inner, 'outer>
  where
    'inner: 'outer;
}

impl<E> Default for Mysql<E> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
  }
}
