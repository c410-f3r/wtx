use crate::misc::{Lease, LeaseMut};

/// Decimal decoder/encoder.
#[derive(Debug, Default)]
pub struct Hex;

impl Lease<Hex> for Hex {
  #[inline]
  fn lease(&self) -> &Hex {
    self
  }
}

impl LeaseMut<Hex> for Hex {
  #[inline]
  fn lease_mut(&mut self) -> &mut Hex {
    self
  }
}
