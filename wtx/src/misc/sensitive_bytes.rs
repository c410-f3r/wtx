use crate::misc::{LeaseMut, memset_slice_volatile, mlock_slice, munlock_slice};
use core::fmt::Debug;

/// Bytes that are zeroed when dropped. See `Secret` for a more confidential container.
pub struct SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  bytes: B,
  is_locked: bool,
}

impl<B> SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  /// New instance ***with*** `mlock`ed bytes
  #[inline]
  pub fn new_locked(mut bytes: B) -> crate::Result<Self> {
    mlock_slice(bytes.lease_mut())?;
    Ok(Self { bytes, is_locked: true })
  }

  /// New instance ***without*** `mlock`ed bytes
  #[inline]
  pub const fn new_unlocked(bytes: B) -> Self {
    Self { bytes, is_locked: false }
  }

  /// Returns an immutable reference to the underlying bytes.
  #[inline]
  pub fn bytes(&self) -> &[u8] {
    self.bytes.lease()
  }

  /// Mutable version of [`SensitiveBytes::bytes`].
  #[inline]
  pub fn bytes_mut(&mut self) -> &mut [u8] {
    self.bytes.lease_mut()
  }
}

impl<B> Debug for SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_tuple("SensitiveBytes").finish()
  }
}

impl<B> Drop for SensitiveBytes<B>
where
  B: LeaseMut<[u8]>,
{
  #[inline]
  fn drop(&mut self) {
    memset_slice_volatile(self.bytes.lease_mut(), 0);
    if self.is_locked {
      let _rslt = munlock_slice(self.bytes.lease_mut());
    }
  }
}
