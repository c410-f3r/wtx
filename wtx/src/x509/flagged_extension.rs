/// An extension with the `critical` flag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlaggedExtension<E> {
  pub(crate) extension: E,
  pub(crate) critical: bool,
}

impl<E> FlaggedExtension<E> {
  pub(crate) const fn new(extension: E, critical: bool) -> Self {
    Self { extension, critical }
  }
}
