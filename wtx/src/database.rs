//! Client connection, schema management and ORM (Object–Relational Mapping).

pub mod client;
mod database_ty;
mod decode;
mod encode;
mod executor;
mod from_record;
mod from_records;
#[cfg(feature = "orm")]
pub mod orm;
mod record;
mod record_values;
mod records;
#[cfg(feature = "sm")]
pub mod sm;
mod stmt;
mod transaction_manager;
mod value;
mod value_ident;

pub use database_ty::DatabaseTy;
pub use decode::Decode;
pub use encode::Encode;
pub use executor::Executor;
pub use from_record::FromRecord;
pub use from_records::FromRecords;
pub use record::Record;
pub use record_values::RecordValues;
pub use records::Records;
pub use stmt::StmtId;
pub use transaction_manager::TransactionManager;
pub use value::Value;
pub use value_ident::ValueIdent;

/// Default environment variable name for the database URL
pub const DEFAULT_URI_VAR: &str = "DATABASE_URI";

/// The maximum number of characters that a database identifier can have. For example, tables,
/// procedures, triggers, etc.
pub type Identifier = arrayvec::ArrayString<64>;
/// Used by some operations to identify different tables
pub type TableSuffix = u32;

/// Database
pub trait Database {
  /// See [DatabaseTy].
  const TY: DatabaseTy;

  /// See [Record].
  type Record<'rec>: Record<Database = Self>;
  /// See [Records].
  type Records<'recs>: Records<Database = Self>;
  /// Representation that can be used to encode or decode types.
  type Value<'value>: Value;
}

impl Database for () {
  const TY: DatabaseTy = DatabaseTy::Unit;

  type Record<'rec> = ();
  type Records<'recs> = ();
  type Value<'value> = ();
}
