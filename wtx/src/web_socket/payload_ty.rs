use crate::misc::{Lease, LeaseMut};

#[derive(Debug)]
pub(crate) enum PayloadTy {
  FirstReader,
  Network,
  None,
  SecondReader,
}

impl Lease<PayloadTy> for PayloadTy {
  #[inline]
  fn lease(&self) -> &PayloadTy {
    self
  }
}

impl LeaseMut<PayloadTy> for PayloadTy {
  #[inline]
  fn lease_mut(&mut self) -> &mut PayloadTy {
    self
  }
}
