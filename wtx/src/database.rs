//! Client connection and schema management.

pub mod client;
mod database_error;
mod database_ty;
mod decode;
mod encode;
mod executor;
mod from_record;
mod from_records;
mod json;
mod misc;
mod record;
mod record_values;
mod records;
#[cfg(feature = "schema-manager")]
pub mod schema_manager;
mod stmt_cmd;
mod transaction_manager;
mod typed;
mod value_ident;

pub use database_error::DatabaseError;
pub use database_ty::DatabaseTy;
pub use decode::Decode;
pub use encode::Encode;
pub use executor::Executor;
pub use from_record::FromRecord;
pub use from_records::FromRecords;
pub use json::Json;
pub use misc::seek_related_entities;
pub use record::Record;
pub(crate) use record_values::encode;
pub use record_values::RecordValues;
pub use records::Records;
pub use stmt_cmd::StmtCmd;
pub use transaction_manager::TransactionManager;
pub use typed::Typed;
pub use value_ident::ValueIdent;

/// The default value for the maximum number of cached statements
pub const DEFAULT_MAX_STMTS: usize = 128;
/// Default environment variable name for the database URL
pub const DEFAULT_URI_VAR: &str = "DATABASE_URI";

/// The maximum number of characters that a database identifier can have. For example, tables,
/// procedures, triggers, etc.
pub type Identifier = crate::misc::ArrayString<64>;

/// Database
pub trait Database {
  /// Prefix used to bind parameterized queries.
  const BIND_PREFIX: &'static str;
  /// Some databases require bindings in ascending order.
  const IS_BIND_INCREASING: bool;
  /// See [`DatabaseTy`].
  const TY: DatabaseTy;

  /// Contains the data used to decode types.
  type DecodeValue<'exec>;
  /// Contains the data used to decode types.
  type EncodeValue<'buffer, 'tmp>: crate::misc::LeaseMut<crate::misc::FilledBufferWriter<'buffer>>
  where
    'buffer: 'tmp;
  /// See [`crate::Error`].
  type Error: From<crate::Error>;
  /// See [Record].
  type Record<'exec>: Record<'exec, Database = Self>;
  /// See [Records].
  type Records<'exec>: Records<'exec, Database = Self>;
  /// All database types
  type Ty;
}

impl Database for () {
  const BIND_PREFIX: &'static str = "$";
  const IS_BIND_INCREASING: bool = true;
  const TY: DatabaseTy = DatabaseTy::Unit;

  type DecodeValue<'exec> = ();
  type EncodeValue<'buffer, 'tmp> = crate::misc::FilledBufferWriter<'buffer>
  where
    'buffer: 'tmp;
  type Error = crate::Error;
  type Record<'exec> = ();
  type Records<'exec> = ();
  type Ty = ();
}
