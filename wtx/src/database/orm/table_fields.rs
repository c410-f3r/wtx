use crate::database::{Database, RecordValues};
use alloc::string::String;

/// Forms all fields of a table.
pub trait TableFields<D>: RecordValues<D>
where
  D: Database,
{
  /// Yields all table field names.
  fn field_names(&self) -> impl Iterator<Item = &'static str>;

  /// Yields all table fields that are or are not optionals.
  fn opt_fields(&self) -> impl Iterator<Item = bool>;

  /// Writes the table instance values for INSERT statements.
  fn write_insert_values(&self, buffer_cmd: &mut String) -> Result<(), D::Error>;

  /// Writes the table instance values for UPDATE statements.
  fn write_update_values(&self, buffer_cmd: &mut String) -> Result<(), D::Error>;
}

impl<D> TableFields<D> for ()
where
  D: Database,
{
  #[inline]
  fn field_names(&self) -> impl Iterator<Item = &'static str> {
    [].into_iter()
  }

  #[inline]
  fn opt_fields(&self) -> impl Iterator<Item = bool> {
    [].into_iter()
  }

  #[inline]
  fn write_insert_values(&self, _: &mut String) -> Result<(), D::Error> {
    Ok(())
  }

  #[inline]
  fn write_update_values(&self, _: &mut String) -> Result<(), D::Error> {
    Ok(())
  }
}
