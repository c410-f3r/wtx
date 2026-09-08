//! Secret

mod encrypted_chunk;
#[cfg(target_os = "linux")]
mod memfd_secret;
mod protected_chunk;
mod secret_data;

use alloc::string::String;
use core::ops::Deref;
pub use encrypted_chunk::{
  EncryptedChunk, EncryptedChunkArray, EncryptedChunkArrayReprSrc, EncryptedChunkSlice,
  EncryptedChunkSliceReprSrc, EncryptedChunkStr, EncryptedChunkStrReprSrc,
};
#[cfg(target_os = "linux")]
pub use memfd_secret::{
  MemFdSecret, MemFdSecretArray, MemFdSecretArrayReprSrc, MemFdSecretSlice,
  MemFdSecretSliceReprSrc, MemFdSecretStr, MemFdSecretStrReprSrc,
};
pub use protected_chunk::ProtectedChunk;
pub use secret_data::SecretData;

/// A `Secret` backed by a array.
pub type SecretArray<const N: usize> = Secret<SecretDataArray<N>>;
/// A `Secret` backed by a slice.
pub type SecretSlice = Secret<SecretDataSlice>;
/// A `Secret` backed by a string slice.
pub type SecretStr = Secret<SecretDataStr>;

cfg_select! {
  target_os = "linux" => {
    type SecretDataArray<const N: usize> = MemFdSecretArray<N>;
    type SecretDataSlice = MemFdSecretSlice;
    type SecretDataStr = MemFdSecretStr;
  }
  _ => {
    type SecretDataArray<const N: usize> = EncryptedChunkArray<N>;
    type SecretDataSlice = EncryptedChunkSlice;
    type SecretDataStr = EncryptedChunkStr;
  }
}

/// Long-lived sensitive data that uses mechanics provided by kernels.
///
/// If desired, you can directly use the `SD` implementations.
#[derive(Debug, Default)]
pub struct Secret<SD>(SD);

impl<SD> Secret<SD>
where
  SD: SecretData,
{
  /// `data` will be internally zeroed regardless if an error occurred.
  #[inline]
  pub fn new(data: &mut <SD::ReprSrc<'_> as Deref>::Target) -> crate::Result<Self> {
    Ok(Self(SD::new(data)?))
  }

  /// Should be dropped as soon as possible and the associated result shouldn't be
  /// cloned into another location. Failing to do so will likely make the usage of this structure
  /// irrelevant and expensive.
  #[inline]
  pub fn peek(&self) -> crate::Result<SD::ReprSrc<'_>> {
    self.0.repr_src()
  }
}

impl TryFrom<String> for SecretStr {
  type Error = crate::Error;

  #[inline]
  fn try_from(mut value: String) -> crate::Result<Self> {
    Self::new(value.as_mut_str())
  }
}

#[cfg(test)]
mod tests {
  use crate::secret::SecretSlice;

  const DATA: [u8; 4] = [1, 2, 3, 4];

  #[cfg_attr(miri, ignore)]
  #[test]
  fn peek() {
    let mut data = DATA;
    let secret = SecretSlice::new(&mut data).unwrap();
    assert_eq!(&*secret.peek().unwrap(), &DATA);
  }
}
