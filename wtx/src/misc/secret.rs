mod secret_context;

use crate::{
  collection::{Clear, TryExtend},
  crypto::{Aead, Aes256GcmRustCrypto, Hash, Sha256DigestRustCrypto},
  misc::{LeaseMut, SensitiveBytes, memset_slice_volatile},
  rng::CryptoRng,
};
use alloc::boxed::Box;
use core::{
  fmt::{Debug, Formatter},
  ops::{Deref, DerefMut},
};
pub use secret_context::SecretContext;

/// Long-lived sensitive data.
///
/// Holds encrypted heap-allocated memory that is decrypted on demand.
///
/// ***Tries*** to provide a layer of protection against Spectre, Meltdown, RowHammer,
/// RAMbleed, etc. Moreover, secrets probably won't be swapped out to the swap area.
///
/// At the current time, does not make use of hardware solutions like TEE.
pub struct Secret {
  protected: Protected,
  salt: [u8; 32],
  secret_context: SecretContext,
}

impl Secret {
  /// `data` will be internally zeroed regardless if an error occurred.
  #[rustfmt::skip]
  pub fn new<RNG: CryptoRng>(
    data: &mut [u8],
    rng: &mut RNG,
    secret_context: SecretContext,
  ) -> crate::Result<Self> {
    let mut data_locked = SensitiveBytes::new_locked(data)?;
    let mut salt = [0; 32];
    rng.fill_slice(&mut salt);
    let (nonce, tag) = {
      let mut secret_key = [0; 32];
      let mut secret_key_locked = SensitiveBytes::new_locked(&mut secret_key)?;
      fill_secret_key(&salt, &secret_context, &mut secret_key_locked)?;
      Aes256GcmRustCrypto::encrypt_in_place_detached(
        &[],
        &mut data_locked,
        rng,
        *secret_key_locked,
      )?
    };
    let all_len = nonce.len().wrapping_add(data_locked.len()).wrapping_add(tag.len());
    let mut protected = Protected::zeroed(all_len);
    if let [
      a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
      content @ ..,
      b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
    ] = &mut *protected {
      copy_iter_mut(&nonce, &mut [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
      copy_iter(&data_locked, content);
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
  pub fn peek<'buffer, B, T>(
    &self,
    buffer: &'buffer mut B,
    fun: impl FnOnce(SensitiveBytes<&'buffer mut [u8]>) -> T,
  ) -> crate::Result<T>
  where
    for<'any> B: Clear + LeaseMut<[u8]> + TryExtend<&'any [u8]>,
  {
    buffer.clear();
    buffer.try_extend(&self.protected)?;
    let mut secret_key = [0; 32];
    let mut secret_key_locked = SensitiveBytes::new_locked(&mut secret_key)?;
    fill_secret_key(&self.salt, &self.secret_context, &mut secret_key_locked)?;
    let data = buffer.lease_mut();
    let plaintext = Aes256GcmRustCrypto::decrypt_in_place(&[], data, *secret_key_locked)?;
    Ok(fun(SensitiveBytes::new_locked(plaintext)?))
  }
}

impl Debug for Secret {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Secret").finish()
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
    unsafe { &*self.0 }
  }
}

impl DerefMut for Protected {
  fn deref_mut(&mut self) -> &mut [u8] {
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

fn fill_secret_key(
  salt: &[u8; 32],
  secret_context: &SecretContext,
  secret_key: &mut SensitiveBytes<&mut [u8; 32]>,
) -> crate::Result<()> {
  let mut array = Sha256DigestRustCrypto::digest(
    [&salt[..]].into_iter().chain(secret_context.0.iter().map(|el| &**el)),
  );
  secret_key.copy_from_slice(&**SensitiveBytes::new_locked(&mut array)?);
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{
    collection::Vector,
    misc::{Secret, SecretContext},
    rng::{ChaCha20, CryptoSeedableRng},
  };

  const DATA: [u8; 4] = [1, 2, 3, 4];

  #[test]
  fn peek() {
    let mut buffer = Vector::new();
    let mut data = DATA;
    let mut rng = ChaCha20::from_std_random().unwrap();
    let secret_context = SecretContext::new(&mut rng).unwrap();
    let secret = Secret::new(&mut data, &mut rng, secret_context).unwrap();
    let mut option = None;
    secret
      .peek(&mut buffer, |local_buffer| {
        option = Some(local_buffer.as_ref().try_into().unwrap());
      })
      .unwrap();
    assert_eq!(option, Some(DATA));
    for elem in &buffer[12..16] {
      assert_eq!(*elem, 0);
    }
  }
}
