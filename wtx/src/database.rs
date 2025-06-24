//! Client connection and schema management.

pub mod client;
mod database_error;
mod database_ty;
mod executor;
mod from_records;
mod json;
mod misc;
mod record;
pub(crate) mod record_values;
mod records;
#[cfg(feature = "schema-manager")]
pub mod schema_manager;
mod stmt_cmd;
mod typed;
mod value_ident;

pub use database_error::DatabaseError;
pub use database_ty::DatabaseTy;
pub use executor::Executor;
pub use from_records::{FromRecords, FromRecordsParams};
pub use json::Json;
pub use misc::seek_related_entities;
pub use record::Record;
pub use record_values::RecordValues;
pub use records::Records;
pub use stmt_cmd::StmtCmd;
pub use typed::{Typed, TypedEncode};
pub use value_ident::ValueIdent;

/// The default value for the maximum number of cached statements
pub const DEFAULT_MAX_STMTS: usize = 128;
/// Default environment variable name for the database URL
pub const DEFAULT_URI_VAR: &str = "DATABASE_URI";

/// The maximum number of characters that a database identifier can have. For example, tables,
/// procedures, triggers, etc.
pub type Identifier = crate::collection::ArrayString<64>;

/// Database
pub trait Database: crate::de::DEController {
  /// See [`DatabaseTy`].
  const TY: DatabaseTy;

  /// See [Record].
  type Record<'exec>: Record<'exec, Database = Self>;
  /// See [Records].
  type Records<'exec>: Records<'exec, Database = Self>;
  /// All database types
  type Ty;
}

impl Database for () {
  const TY: DatabaseTy = DatabaseTy::Unit;

  type Record<'exec> = ();
  type Records<'exec> = ();
  type Ty = ();
}
