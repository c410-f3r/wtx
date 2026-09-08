use core::ops::Deref;

/// Secret data
pub trait SecretData: Sized {
  /// Representation source
  type ReprSrc<'this>: Deref;

  /// New instance
  fn new(data: &mut <Self::ReprSrc<'_> as Deref>::Target) -> crate::Result<Self>;

  /// Representation source
  fn repr_src(&self) -> crate::Result<Self::ReprSrc<'_>>;
}
