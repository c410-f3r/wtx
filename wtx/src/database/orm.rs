//! # Object–Relational Mapping

mod crud;
mod full_table_association;
mod misc;
mod no_table_association;
mod select_limit;
mod select_order_by;
mod sql_value;
mod sql_writer;
mod table;
mod table_association;
mod table_association_wrapper;
mod table_associations;
mod table_field;
mod table_fields;
mod table_params;
mod table_source_association;
#[cfg(test)]
mod tests;
mod tuple_impls;

pub use crud::Crud;
pub use full_table_association::*;
pub use misc::*;
pub use no_table_association::*;
pub use select_limit::*;
pub use select_order_by::*;
pub use sql_value::*;
pub use sql_writer::*;
pub use table::*;
pub use table_association::*;
pub use table_association_wrapper::*;
pub use table_associations::*;
pub use table_field::*;
pub use table_fields::*;
pub use table_params::*;
pub use table_source_association::*;

/// Shortcut to avoid having to manually type the result of [Table::new]
pub type FromSuffixRslt<'ent, T> = (<T as Table<'ent>>::Associations, <T as Table<'ent>>::Fields);
/// Used by initial calls of [SqlWriter::write_insert]
pub type InitialInsertValue = i32;

pub(crate) type AuxNodes = smallvec::SmallVec<[(u64, &'static str); 32]>;
