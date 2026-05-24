/// An extension with the `critical` flag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlaggedExtension<E> {
  extension: E,
  critical: bool,
}

impl<E> FlaggedExtension<E> {
  pub(crate) const fn new(extension: E, critical: bool) -> Self {
    Self { extension, critical }
  }

  /// Extension
  #[inline]
  pub const fn extension(&self) -> &E {
    &self.extension
  }

  /// Is a critical extension
  #[inline]
  pub const fn critical(&self) -> bool {
    self.critical
  }
}
