use crate::misc::{Lease, LeaseMut};

/// Type that indicates the usage of the `quick-protobuf` dependency.
#[derive(Debug, Default)]
pub struct QuickProtobuf;

impl Lease<QuickProtobuf> for QuickProtobuf {
  #[inline]
  fn lease(&self) -> &QuickProtobuf {
    self
  }
}

impl LeaseMut<QuickProtobuf> for QuickProtobuf {
  #[inline]
  fn lease_mut(&mut self) -> &mut QuickProtobuf {
    self
  }
}
