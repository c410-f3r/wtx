use crate::misc::{Lease, LeaseMut};

/// Type that indicates messages encoded or decoded with percent-encoding
#[derive(Debug, Default)]
pub struct Urlencoded;

impl Lease<Urlencoded> for Urlencoded {
  #[inline]
  fn lease(&self) -> &Urlencoded {
    self
  }
}

impl LeaseMut<Urlencoded> for Urlencoded {
  #[inline]
  fn lease_mut(&mut self) -> &mut Urlencoded {
    self
  }
}
