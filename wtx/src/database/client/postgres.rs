//! Postgres, is a free and open-source relational database management system (RDBMS) emphasizing
//! extensibility and SQL compliance.

mod authentication;
mod config;
mod db_error;
mod executor;
mod executor_buffer;
mod field;
#[cfg(all(feature = "_integration-tests", test))]
mod integration_tests;
mod message;
mod protocol;
mod record;
mod records;
mod sql_state;
mod statements;
mod transaction_manager;
mod ty;
mod tys;
mod value;

use core::marker::PhantomData;

use crate::database::{Database, DatabaseTy};
pub(crate) use authentication::Authentication;
pub use config::Config;
pub use db_error::{DbError, ErrorPosition};
pub use executor::Executor;
pub use executor_buffer::ExecutorBuffer;
pub(crate) use field::MsgField;
pub(crate) use message::MessageTy;
pub(crate) use protocol::*;
pub use record::Record;
pub use records::Records;
pub use sql_state::SqlState;
pub use statements::Statements;
pub use transaction_manager::TransactionManager;
pub use value::Value;

pub(crate) type Oid = u32;

/// Postgres
#[derive(Debug)]
pub struct Postgres<E>(PhantomData<E>);

impl<E> Default for Postgres<E> {
  #[inline]
  fn default() -> Self {
    Self(PhantomData)
  }
}

impl<E> Database for Postgres<E>
where
  E: From<crate::Error>,
{
  const TY: DatabaseTy = DatabaseTy::Postgres;

  type Error = E;
  type Record<'rec> = Record<'rec, E>;
  type Records<'recs> = Records<'recs, E>;
  type Value<'value> = Value<'value>;
}
