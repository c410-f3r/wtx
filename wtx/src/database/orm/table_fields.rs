use alloc::string::String;
use core::array;

/// Forms all fields of a table.
pub trait TableFields<E> {
  /// Iterator of fields.
  type FieldNames: Iterator<Item = &'static str>;

  /// Yields all table field names.
  fn field_names(&self) -> Self::FieldNames;

  /// Writes the table instance values for INSERT statements.
  fn write_insert_values(&self, buffer_cmd: &mut String) -> Result<(), E>;

  /// Writes the table instance values for UPDATE statements.
  fn write_update_values(&self, buffer_cmd: &mut String) -> Result<(), E>;
}

impl<E> TableFields<E> for ()
where
  E: From<crate::Error>,
{
  type FieldNames = array::IntoIter<&'static str, 0>;

  #[inline]
  fn field_names(&self) -> Self::FieldNames {
    [].into_iter()
  }

  #[inline]
  fn write_insert_values(&self, _: &mut String) -> Result<(), E> {
    Ok(())
  }

  #[inline]
  fn write_update_values(&self, _: &mut String) -> Result<(), E> {
    Ok(())
  }
}
