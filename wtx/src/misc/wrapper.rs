use crate::misc::{Lease, LeaseMut};

/// Wrapper used to work around coherence rules.
#[derive(Debug)]
pub struct Wrapper<T>(
  /// Element
  pub T,
);

impl<T> Lease<T> for Wrapper<T> {
  #[inline]
  fn lease(&self) -> &T {
    &self.0
  }
}

impl<T> LeaseMut<T> for Wrapper<T> {
  #[inline]
  fn lease_mut(&mut self) -> &mut T {
    &mut self.0
  }
}

impl<T> Lease<Wrapper<T>> for Wrapper<T> {
  #[inline]
  fn lease(&self) -> &Wrapper<T> {
    self
  }
}

impl<T> LeaseMut<Wrapper<T>> for Wrapper<T> {
  #[inline]
  fn lease_mut(&mut self) -> &mut Wrapper<T> {
    self
  }
}
