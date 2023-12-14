/// Table field name and its associated Rust type
#[derive(Debug, PartialEq)]
pub struct TableField<T> {
  name: &'static str,
  value: Option<T>,
}

impl<T> TableField<T> {
  /// Creates a new instance from the table field name
  #[inline]
  pub const fn new(name: &'static str) -> Self {
    Self { name, value: None }
  }

  /// Table field name
  #[inline]
  pub const fn name(&self) -> &'static str {
    self.name
  }

  /// Table field value
  #[inline]
  pub const fn value(&self) -> &Option<T> {
    &self.value
  }

  /// Mutable version of [value]
  #[inline]
  pub fn value_mut(&mut self) -> &mut Option<T> {
    &mut self.value
  }
}
