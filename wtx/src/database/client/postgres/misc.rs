use crate::database::{
  Identifier,
  client::postgres::{Ty, postgres_column_info::PostgresColumnInfo},
};

pub(crate) const fn dummy_stmt_value() -> (PostgresColumnInfo, Ty) {
  (PostgresColumnInfo::new(Identifier::new(), Ty::Any), Ty::Any)
}
