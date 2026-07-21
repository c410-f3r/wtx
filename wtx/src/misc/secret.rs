use crate::{
  collections::{SingleTypeStorage, SuffixGuard, Truncate, TryExtend},
  crypto::{Aead as _, Aes256GcmGlobal, gen_aead_nonce},
  misc::{LeaseMut, SensitiveBytes, memset_slice_volatile},
  rng::CryptoRng,
  sync::Arc,
};
use alloc::boxed::Box;
use core::{
  fmt::{Debug, Formatter},
  ops::{Deref, DerefMut},
};

const CTX_LEN: usize = cfg_select! {
  target_pointer_width = "64" => 4096,
  _ => 2048
};

/// Long-lived sensitive data.
///
/// Holds encrypted heap-allocated memory that is decrypted on demand.
///
/// ***Tries*** to provide a layer of protection against Spectre, Meltdown, `RowHammer`,
/// `RAMbleed`, etc. Moreover, secrets probably won't be swapped out to the swap area.
///
/// At the current time, does not make use of hardware solutions like TEE.
pub struct Secret {
  protected: Protected,
  salt: [u8; 32],
  secret_context: SecretContext,
}

impl Secret {
  /// `data` will be internally zeroed regardless if an error occurred.
  #[inline]
  #[rustfmt::skip]
  pub fn new<RNG>(
    data: &mut [u8],
    rng: &mut RNG,
    secret_context: SecretContext,
  ) -> crate::Result<Self>
  where
    RNG: CryptoRng
  {
    let mut data_wrapper = SensitiveBytes::new(data);
    let mut salt = [0; 32];
    rng.fill_slice(&mut salt);
    let nonce = gen_aead_nonce(rng);
    let tag =  Aes256GcmGlobal::encrypt_parts(
      &[],
      nonce,
      &mut data_wrapper,
      gen_secret_key(&salt, &secret_context).as_bytes(),
    )?;
    let all_len = nonce.len().wrapping_add(data_wrapper.len()).wrapping_add(tag.len());
    let mut protected = Protected::zeroed(all_len);
    if let [
      a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
      content @ ..,
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
    ] = &mut *protected {
      copy_iter_mut(&nonce, &mut [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
      copy_iter(&data_wrapper, content);
      copy_iter_mut(&tag, &mut [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15]);
    }
    Ok(Self { protected, salt, secret_context })
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
  pub fn peek<'buffer, B, T>(
    &self,
    buffer: &'buffer mut SuffixGuard<B>,
    fun: impl FnOnce(SensitiveBytes<&'buffer mut [u8]>) -> T,
  ) -> crate::Result<T>
  where
    for<'any> B:
      LeaseMut<[u8]> + SingleTypeStorage<Item = u8> + Truncate<usize> + TryExtend<&'any [u8]>,
  {
    buffer.inner_mut().try_extend(&self.protected)?;
    let plaintext = Aes256GcmGlobal::decrypt_in_place(
      &[],
      buffer.curr_mut(),
      gen_secret_key(&self.salt, &self.secret_context).as_bytes(),
    )?;
    Ok(fun(SensitiveBytes::new(plaintext)))
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
      salt: [0; 32],
      secret_context: SecretContext::default(),
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

impl Default for SecretContext {
  #[inline]
  fn default() -> Self {
    Self(Arc::new(Protected::zeroed(0)))
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

fn gen_secret_key(salt: &[u8; 32], secret_context: &SecretContext) -> blake3::Hash {
  let mut hasher = blake3::Hasher::new();
  let _ = hasher.update(&salt[..]).update(&secret_context.0);
  hasher.finalize()
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
      .peek(&mut buffer.into(), |local_buffer| {
        option = Some(local_buffer.as_ref().try_into().unwrap());
      })
      .unwrap();
    assert_eq!(option, Some(DATA));
    assert_eq!(buffer.len(), 0);
  }
}
