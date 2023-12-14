/// Used by nodes that need source (backward) information
#[derive(Clone, Copy, Debug)]
pub struct TableSourceAssociation<'value, V = &'static str> {
  source_field: &'static str,
  source_value: &'value V,
}

impl<'value, V> TableSourceAssociation<'value, V> {
  #[inline]
  pub(crate) const fn new(source_value: &'value V) -> Self {
    Self { source_field: "", source_value }
  }

  #[inline]
  pub(crate) const fn source_field(&self) -> &'static str {
    self.source_field
  }

  #[inline]
  pub(crate) fn source_field_mut(&mut self) -> &mut &'static str {
    &mut self.source_field
  }

  #[inline]
  pub(crate) const fn source_value(&self) -> &'value V {
    self.source_value
  }
}
