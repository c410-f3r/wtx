use crate::crypto::HashTy;

/// Signing output
#[derive(Debug)]
pub struct SigningOutput<S> {
  hash_ty: HashTy,
  signature: S,
}

impl<S> SigningOutput<S> {
  /// New instance that uses a default hashing algorithm.
  ///
  /// It is OK to use this method for anything that is not RSA.
  #[inline]
  pub fn from_signature(signature: S) -> Self {
    Self { hash_ty: HashTy::default(), signature }
  }

  /// New instance
  #[inline]
  pub const fn new(hash_ty: HashTy, signature: S) -> Self {
    Self { hash_ty, signature }
  }

  /// Converts `S` into bytes.
  #[inline]
  pub fn as_bytes(&self) -> SigningOutput<&[u8]>
  where
    S: AsRef<[u8]>,
  {
    SigningOutput::new(self.hash_ty, self.signature.as_ref())
  }

  /// See [`HashTy`].
  #[inline]
  pub const fn hash_ty(&self) -> HashTy {
    self.hash_ty
  }

  /// Checkout the documentation of the chose signature data type.
  #[inline]
  pub const fn signature(&self) -> &S {
    &self.signature
  }

  /// Mutable version of [`Self::signature`].
  #[inline]
  pub const fn signature_mut(&mut self) -> &mut S {
    &mut self.signature
  }

  /// Owned version of [`Self::signature`].
  #[inline]
  pub fn into_signature(self) -> S {
    self.signature
  }
}
