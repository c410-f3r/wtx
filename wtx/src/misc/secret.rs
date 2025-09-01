use crate::{collection::Vector, misc::memset_slice_volatile};
use alloc::boxed::Box;
use core::{
  fmt::{Debug, Formatter},
  ops::{Deref, DerefMut},
};

/// Long-lived sensitive data.
///
/// Holds encrypted heap-allocated memory that is decrypted on demand. ***Tries*** to provide a
/// layer of protection against Spectre, Meltdown, Rowhammer, RAMbleed and coldboot attacks. Moreover, data
/// swapped out to the swap area ***probably*** should not be a problem.
pub struct Secret {
  protected: Protected,
  salt: [u8; 32],
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
    Vector::from_vec(alloc::vec![0; size]).into_vec().into_boxed_slice().into()
  }
}

impl Clone for Protected {
  fn clone(&self) -> Self {
    let mut vec = alloc::vec::Vec::with_capacity(self.len());
    vec.extend_from_slice(self);
    vec.into_boxed_slice().into()
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
    Protected(Box::leak(from))
  }
}

// SAFETY: Inner pointer is unique
unsafe impl Send for Protected {}
// SAFETY: Inner pointer is unique
unsafe impl Sync for Protected {}

fn copy_iter(from: &[u8], to: &mut [u8]) {
  from.iter().zip(to.iter_mut()).for_each(|(lhs, rhs)| *rhs = *lhs);
}

mod static_keys {
  use crate::{
    collection::{ExpansionTy, Vector},
    misc::{
      Secret, decrypt_aes256gcm, encrypt_aes256gcm_vectored, memset_slice_volatile,
      secret::{Protected, copy_iter},
    },
    rng::CryptoRng,
  };
  use alloc::boxed::Box;
  use sha2::{Digest, Sha256};
  use std::sync::OnceLock;

  const STATIC_KEYS_NUM: usize = 4;
  const STATIC_KEYS_SIZE: usize = 4096;

  static STATIC_KEYS: OnceLock<Box<[Box<[u8]>]>> = OnceLock::new();

  impl Secret {
    /// `data` will be internally zeroed regardless if an error occurred.
    pub fn new<RNG: CryptoRng>(data: &mut [u8], rng: &mut RNG) -> crate::Result<Self> {
      let rslt = Self::do_new(data, rng);
      memset_slice_volatile(data, 0);
      rslt
    }

    /// Decrypts secret  
    pub fn peek<T>(&self, mut fun: impl FnMut(&[u8]) -> T) -> crate::Result<T> {
      Ok(fun(decrypt_aes256gcm(
        &[],
        &mut self.protected.clone(),
        &secret_key(&self.salt)?.as_ref().try_into()?,
      )?))
    }

    #[rustfmt::skip]
    fn do_new<RNG>(data: &mut [u8], rng: &mut RNG) -> crate::Result<Self>
    where
      RNG: CryptoRng,
    {
      let _rslt = STATIC_KEYS.set({
        let mut pages = Vector::new();
        for _ in 0..STATIC_KEYS_NUM {
          let mut page = Vector::with_capacity(STATIC_KEYS_SIZE)?;
          // SAFETY: pointer comes from allocated memory
          #[cfg(not(miri))]
          unsafe {
            let capacity = page.capacity();
            crate::misc::mlock(page.as_ptr_mut(), capacity)?;
          }
          page.expand(ExpansionTy::Len(STATIC_KEYS_SIZE), 0)?;
          rng.fill_slice(&mut page);
          pages.push(page.into_vec().into())?;
        }
        pages.into_vec().into()
      });
      let mut salt = [0; 32];
      rng.fill_slice(&mut salt);
      let secret_key = secret_key(&salt)?.as_ref().try_into()?;
      let (nonce, tag) = encrypt_aes256gcm_vectored(&[], data, &secret_key, rng)?;
      let all_len = nonce.len().wrapping_add(data.len()).wrapping_add(tag.len());
      let mut protected = Protected::zeroed(all_len);
      if let [
        a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11,
        content @ ..,
        b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15
      ] = &mut *protected {
        copy_iter_mut(&nonce, &mut [a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
        copy_iter(data, content);
        copy_iter_mut(&tag, &mut [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15]);
      }
      Ok(Self { protected, salt })
    }
  }

  fn copy_iter_mut(from: &[u8], to: &mut [&mut u8]) {
    from.iter().zip(to.iter_mut()).for_each(|(lhs, rhs)| **rhs = *lhs);
  }

  fn secret_key(salt: &[u8; 32]) -> crate::Result<Protected> {
    let mut ctx = Sha256::new();
    ctx.update(salt);
    STATIC_KEYS.wait().iter().for_each(|static_key| ctx.update(static_key));
    Ok(Protected::from(<[u8; 32]>::from(ctx.finalize()).as_ref()))
  }
}

#[cfg(test)]
mod tests {
  use crate::{misc::Secret, rng::ChaCha20, tests::_32_bytes_seed};

  const DATA: [u8; 4] = [1, 2, 3, 4];

  #[test]
  fn peek() {
    let mut data = DATA;
    let secret = Secret::new(&mut data, &mut ChaCha20::new(_32_bytes_seed())).unwrap();
    assert_eq!(data, [0, 0, 0, 0]);
    let mut option = None;
    secret
      .peek(|bytes| {
        option = Some(bytes.try_into().unwrap());
      })
      .unwrap();
    assert_eq!(option, Some(DATA));
  }
}
