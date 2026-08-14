use crate::{
  collections::{SingleTypeStorage, SuffixGuard, Truncate, TryExtend},
  crypto::{Aead as _, Aes128GcmGlobal, gen_aead_nonce},
  misc::{LeaseMut, SensitiveBytes, memset_slice_volatile},
  rng::CryptoRng,
  sync::Arc,
};
use alloc::boxed::Box;
use core::{
  fmt::{Debug, Formatter},
  ops::{Deref, DerefMut},
};

const CTX_LEN: usize = 1024;
const SECRET_LEN: usize = 16;

/// Long-lived sensitive data.
///
/// Holds encrypted heap-allocated memory that is decrypted on demand.
///
/// ***Tries*** to provide a layer of protection against Spectre, Meltdown, `RowHammer`,
/// `RAMbleed`, etc.
pub struct Secret {
  protected: Protected,
  salt: [u8; SECRET_LEN],
  secret_context: SecretContext,
}

impl Secret {
  /// `data` will be internally zeroed regardless if an error occurred.
  #[inline]
  pub fn new<RNG>(
    data: &mut [u8],
    rng: &mut RNG,
    secret_context: SecretContext,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut data_wrapper = SensitiveBytes::new(data);
    let mut salt = [0; SECRET_LEN];
    rng.fill_slice(&mut salt);
    let nonce = gen_aead_nonce(rng);
    let tag = Aes128GcmGlobal::encrypt_parts(
      &[],
      nonce,
      &mut data_wrapper,
      &gen_secret_key(&salt, &secret_context),
    )?;
    Ok(Self { protected: gen_protected(&data_wrapper, nonce, tag), salt, secret_context })
  }

  /// Decrypts secret temporally.
  ///
  /// The bytes of the closure shouldn't be cloned into another location. Failing to do so
  /// will likely make the usage of this structure irrelevant and expensive.
  ///
  /// `buffer` is utilized for internal operations and can be freely reused for any other action
  /// afterwards. Please note that its capacity should at least be the original data byte length
  /// plus 28 bytes.
  ///
  /// When the closure is executing, the plaintext secret will exist transiently in CPU registers
  /// and caches, which is unavoidable.
  #[inline]
  pub fn peek<'buffer, 'sp, 'this, B, T>(
    &'this self,
    buffer: &'buffer mut SuffixGuard<B>,
    fun: impl FnOnce(SecretPeek<'sp>) -> T,
  ) -> crate::Result<T>
  where
    'buffer: 'sp,
    'this: 'sp,
    for<'any> B:
      LeaseMut<[u8]> + SingleTypeStorage<Item = u8> + Truncate<usize> + TryExtend<&'any [u8]>,
  {
    buffer.inner_mut().try_extend(&self.protected)?;
    let plaintext = Aes128GcmGlobal::decrypt_in_place(
      &[],
      buffer.curr_mut(),
      &gen_secret_key(&self.salt, &self.secret_context),
    )?;
    Ok(fun(SecretPeek(SensitiveBytes::new(plaintext))))
  }
}

impl Debug for Secret {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Secret").finish()
  }
}

impl Default for Secret {
  #[inline]
  fn default() -> Self {
    Self {
      protected: Protected::zeroed(0),
      salt: [0; SECRET_LEN],
      secret_context: SecretContext(Arc::new(Protected::zeroed(0))),
    }
  }
}

/// Used by `Secret`, can be freely cloned and shared across threads.
#[derive(Clone)]
pub struct SecretContext(Arc<Protected>);

impl SecretContext {
  /// New instance
  #[inline]
  pub fn new<RNG>(rng: &mut RNG) -> crate::Result<Self>
  where
    RNG: CryptoRng,
  {
    let mut protected = Protected::zeroed(CTX_LEN);
    rng.fill_slice(&mut protected);
    #[cfg(feature = "libc")]
    crate::misc::mlock_slice(&mut protected)?;
    Ok(Self(Arc::new(protected)))
  }
}

impl Debug for SecretContext {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SecretContext").finish()
  }
}

/// Element returned by [`Secret::peek`].
pub struct SecretPeek<'any>(SensitiveBytes<&'any mut [u8]>);

impl<'any> SecretPeek<'any> {
  /// Inner content
  #[inline]
  pub fn data(&'any self) -> &'any [u8] {
    &self.0
  }
}

impl Debug for SecretPeek<'_> {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("SecretPeek").finish()
  }
}

// A chunk of heap-allocated memory that is zeroed when dropped. The use of a pointer
// prevents compiler optimizations
struct Protected(*mut [u8]);

impl Protected {
  fn zeroed(size: usize) -> Protected {
    alloc::vec![0; size].into_boxed_slice().into()
  }
}

impl Debug for Protected {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Protected").finish()
  }
}

impl Deref for Protected {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    // SAFETY: Pointer comes from a valid owned chunk of memory according to all related
    //         constructors
    unsafe { &*self.0 }
  }
}

impl DerefMut for Protected {
  fn deref_mut(&mut self) -> &mut [u8] {
    // SAFETY: Pointer comes from a valid owned chunk of memory according to all related
    //         constructors
    unsafe { &mut *self.0 }
  }
}

impl Drop for Protected {
  fn drop(&mut self) {
    memset_slice_volatile(self, 0);
    #[cfg(feature = "libc")]
    let _rslt = crate::misc::munlock_slice(self);
    // SAFETY: Instance has a valid allocated chunk of memory
    unsafe {
      drop(Box::from_raw(self.0));
    }
  }
}

impl From<&[u8]> for Protected {
  fn from(from: &[u8]) -> Self {
    let mut protected = Protected::zeroed(from.len());
    copy_iter(from, &mut protected);
    protected
  }
}

impl From<Box<[u8]>> for Protected {
  fn from(from: Box<[u8]>) -> Self {
    Protected(Box::into_raw(from))
  }
}

// SAFETY: Inner pointer is unique
unsafe impl Send for Protected {}
// SAFETY: Inner pointer is unique
unsafe impl Sync for Protected {}

fn copy_iter(from: &[u8], to: &mut [u8]) {
  from.iter().zip(to.iter_mut()).for_each(|(lhs, rhs)| *rhs = *lhs);
}

fn copy_iter_mut(from: &[u8], to: &mut [&mut u8]) {
  from.iter().zip(to.iter_mut()).for_each(|(lhs, rhs)| **rhs = *lhs);
}

#[inline]
#[rustfmt::skip]
fn gen_protected(
  encrypted: &SensitiveBytes<&mut [u8]>,
  nonce: [u8; 12],
  tag: [u8; 16]
) -> Protected {
  let all_len = nonce.len().wrapping_add(encrypted.len()).wrapping_add(tag.len());
  let mut protected = Protected::zeroed(all_len);
  if let [
    a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
    content @ ..,
    b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
  ] = &mut *protected {
    copy_iter_mut(&nonce, &mut [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
    copy_iter(encrypted, content);
    copy_iter_mut(&tag, &mut [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15]);
  }
  protected
}

fn gen_secret_key(salt: &[u8; SECRET_LEN], secret_context: &SecretContext) -> [u8; SECRET_LEN] {
  let mut hasher = blake3::Hasher::new();
  let _ = hasher.update(&salt[..]).update(&secret_context.0);
  let mut rslt = [0; SECRET_LEN];
  hasher.finalize_xof().fill(&mut rslt);
  rslt
}

#[cfg(test)]
mod tests {
  use crate::{
    collections::Vector,
    misc::{Secret, SecretContext},
    rng::{ChaCha20, CryptoSeedableRng},
  };

  const DATA: [u8; 4] = [1, 2, 3, 4];

  #[cfg_attr(miri, ignore)]
  #[test]
  fn peek() {
    let buffer = &mut Vector::new();
    let mut data = DATA;
    let mut rng = ChaCha20::from_std_random().unwrap();
    let secret_context = SecretContext::new(&mut rng).unwrap();
    let secret = Secret::new(&mut data, &mut rng, secret_context).unwrap();
    let mut option = None;
    secret
      .peek(&mut buffer.into(), |sp| {
        option = Some(sp.data().try_into().unwrap());
      })
      .unwrap();
    assert_eq!(option, Some(DATA));
    assert_eq!(buffer.len(), 0);
  }
}
