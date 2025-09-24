use crate::misc::memset_slice_volatile;
use core::fmt::Debug;

/// Bytes that are zeroed when dropped. See `Secret` for a more confidential container.
pub struct SensitiveBytes<B>(
  /// Bytes
  pub B,
)
where
  B: AsMut<[u8]>;

impl<B> Debug for SensitiveBytes<B>
where
  B: AsMut<[u8]>,
{
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_tuple("SensitiveBytes").finish()
  }
}

impl<B> Drop for SensitiveBytes<B>
where
  B: AsMut<[u8]>,
{
  #[inline]
  fn drop(&mut self) {
    memset_slice_volatile(self.0.as_mut(), 0);
  }
}
