//! Postgres, is a free and open-source relational database management system (RDBMS) emphasizing
//! extensibility and SQL compliance.

mod authentication;
mod config;
mod db_error;
mod decode_value;
mod encode_value;
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
mod transaction_manager;
mod ty;
mod tys;

use crate::database::{Database, DatabaseTy};
pub use config::Config;
use core::marker::PhantomData;
pub use db_error::{DbError, ErrorPosition, Severity};
pub use decode_value::DecodeValue;
pub use encode_value::EncodeValue;
pub use executor::Executor;
pub use executor_buffer::ExecutorBuffer;
pub use postgres_error::PostgresError;
pub use record::Record;
pub use records::Records;
pub use sql_state::SqlState;
pub use statements::Statements;
pub use struct_decoder::StructDecoder;
pub use struct_encoder::StructEncoder;
pub use transaction_manager::TransactionManager;
pub use ty::Ty;

pub(crate) type Oid = u32;

/// Postgres
#[derive(Debug)]
pub struct Postgres<E>(PhantomData<fn() -> E>);

impl<E> Database for Postgres<E>
where
  E: From<crate::Error>,
{
  const BIND_PREFIX: &'static str = "$";
  const IS_BIND_INCREASING: bool = true;
  const TY: DatabaseTy = DatabaseTy::Postgres;

  type DecodeValue<'exec> = DecodeValue<'exec>;
  type EncodeValue<'buffer, 'tmp> = EncodeValue<'buffer, 'tmp>
  where
    'buffer: 'tmp;
  type Error = E;
  type Record<'exec> = Record<'exec, E>;
  type Records<'exec> = Records<'exec, E>;
  type Ty = Ty;
}

impl<E> Default for Postgres<E> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
  }
}

#[cfg(test)]
mod tests {
  use crate::database::client::postgres::{statements::Column, Ty};

  pub(crate) fn column0() -> Column {
    Column { name: "a".try_into().unwrap(), ty: Ty::VarcharArray }
  }

  pub(crate) fn column1() -> Column {
    Column { name: "b".try_into().unwrap(), ty: Ty::Int8 }
  }

  pub(crate) fn column2() -> Column {
    Column { name: "c".try_into().unwrap(), ty: Ty::Char }
  }

  pub(crate) fn column3() -> Column {
    Column { name: "d".try_into().unwrap(), ty: Ty::Date }
  }
}
