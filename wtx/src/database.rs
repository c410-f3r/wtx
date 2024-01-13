//! Client connection, schema manager and ORM (Object–Relational Mapping).

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
#[cfg(feature = "schema-manager")]
pub mod schema_manager;
mod stmt_cmd;
mod transaction_manager;
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
pub use stmt_cmd::StmtCmd;
pub use transaction_manager::TransactionManager;
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

  /// Contains the data used to decode types.
  type DecodeValue<'dv>;
  /// Contains the data used to decode types.
  type EncodeValue<'ev>;
  /// See [crate::Error].
  type Error: From<crate::Error>;
  /// See [Record].
  type Record<'rec>: Record<Database = Self>;
  /// See [Records].
  type Records<'recs>: Records<Database = Self>;
}

impl Database for () {
  const TY: DatabaseTy = DatabaseTy::Unit;

  type DecodeValue<'dv> = ();
  type EncodeValue<'ev> = ();
  type Error = crate::Error;
  type Record<'rec> = ();
  type Records<'recs> = ();
}
