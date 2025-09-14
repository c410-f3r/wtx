use core::fmt::Debug;

use crate::misc::memset_slice_volatile;

/// Bytes that are zeroed when dropped. See `Secret` for a more confidential container.
pub struct SensitiveBytes<'bytes>(
  /// Bytes
  pub &'bytes mut [u8],
);

impl Drop for SensitiveBytes<'_> {
  #[inline]
  fn drop(&mut self) {
    memset_slice_volatile(self.0, 0);
  }
}

impl Debug for SensitiveBytes<'_> {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_tuple("SensitiveBytes").finish()
  }
}
