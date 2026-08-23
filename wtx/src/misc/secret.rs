#[cfg(not(all(feature = "libc", target_os = "linux")))]
mod encrypted;
#[cfg(all(feature = "libc", target_os = "linux"))]
mod memfd_secret;
mod protected;
mod secret_context;

use crate::{
  collections::{Truncate, TryExtend},
  misc::LeaseMut,
  rng::CryptoRng,
};
use core::{
  fmt::{Debug, Formatter},
  range::Range,
};
pub use secret_context::SecretContext;

/// Long-lived sensitive data.
///
/// ***Tries*** to provide a layer of protection against Spectre, Meltdown, `RowHammer`,
/// `RAMbleed`, etc.
#[derive(Default)]
pub struct Secret {
  #[cfg(all(feature = "libc", target_os = "linux"))]
  inner: memfd_secret::MemFdSecret,
  #[cfg(not(all(feature = "libc", target_os = "linux")))]
  inner: encrypted::Encrypted,
}

impl Secret {
  /// `data` will be internally zeroed regardless if an error occurred.
  #[inline]
  pub fn new<RNG>(
    data: &mut [u8],
    _rng: &mut RNG,
    _secret_context: SecretContext,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let inner = cfg_select! {
      all(feature = "libc", target_os = "linux") => {{
        let opt = memfd_secret::MemFdSecret::new(data, true);
        opt.ok_or(crate::Error::UnsupportedLinuxKernel)?
      }},
      _ => encrypted::Encrypted::new(data, _rng, _secret_context)?
    };
    Ok(Self { inner })
  }

  /// [`SecretPeek`] should be dropped as soon as possible and the associated bytes shouldn't be
  /// cloned into another location. Failing to do so will likely make the usage of this structure
  /// irrelevant and expensive.
  ///
  /// `buffer` is utilized for internal operations and can be freely reused for any other action
  /// afterwards.
  ///
  /// While [`SecretPeek`] is alive the plaintext secret will exist transiently in CPU registers
  /// and caches, which is unavoidable.
  #[inline]
  pub fn peek<'buffer, 'sp, 'this, B>(
    &'this self,
    buffer: &'buffer mut B,
  ) -> crate::Result<SecretPeek<'sp, B>>
  where
    'buffer: 'sp,
    'this: 'sp,
    for<'any> B: LeaseMut<[u8]> + Truncate<usize> + TryExtend<&'any [u8]>,
  {
    let (_idx, _range) = cfg_select! {
      all(feature = "libc", target_os = "linux") => (0, Range::default()),
      _ => self.inner.peek(buffer)?
    };
    Ok(SecretPeek { _buffer: buffer, _idx, _range, _this: self })
  }
}

impl Debug for Secret {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Secret").finish()
  }
}

/// Element returned by [`Secret::peek`].
pub struct SecretPeek<'any, B>
where
  B: LeaseMut<[u8]> + Truncate<usize>,
{
  _buffer: &'any mut B,
  _idx: usize,
  _range: Range<usize>,
  _this: &'any Secret,
}

impl<B> SecretPeek<'_, B>
where
  B: LeaseMut<[u8]> + Truncate<usize>,
{
  /// Inner content
  #[inline]
  pub fn data(&self) -> &[u8] {
    cfg_select! {
      all(feature = "libc", target_os = "linux") => &self._this.inner,
      _ => self._buffer.lease()
        .get(self._idx..)
        .and_then(|slice| slice.get(self._range))
        .unwrap_or_default()
    }
  }
}

impl<B> Debug for SecretPeek<'_, B>
where
  B: LeaseMut<[u8]> + Truncate<usize>,
{
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SecretPeek").finish()
  }
}

#[cfg(not(all(feature = "libc", target_os = "linux")))]
impl<B> Drop for SecretPeek<'_, B>
where
  B: LeaseMut<[u8]> + Truncate<usize>,
{
  #[inline]
  fn drop(&mut self) {
    drop(crate::misc::SensitiveBytes::new(
      self._buffer.lease_mut().get_mut(self._idx..).unwrap_or_default(),
    ));
    self._buffer.truncate(self._idx);
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    collections::Vector,
    misc::{SecretContext, secret::Secret},
    rng::{ChaCha20, CryptoSeedableRng},
  };

  const DATA: [u8; 4] = [1, 2, 3, 4];

  #[cfg_attr(miri, ignore)]
  #[test]
  fn peek() {
    let mut buffer = Vector::new();
    let mut data = DATA;
    let mut rng = ChaCha20::from_std_random().unwrap();
    let secret_context = SecretContext::new(&mut rng).unwrap();
    let secret = Secret::new(&mut data, &mut rng, secret_context).unwrap();
    {
      let bytes = secret.peek(&mut buffer).unwrap();
      assert_eq!(Some(bytes.data().try_into().unwrap()), Some(DATA));
    }
    assert_eq!(buffer.len(), 0);
  }
}
