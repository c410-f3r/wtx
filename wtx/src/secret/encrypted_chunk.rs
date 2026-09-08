use crate::{
  collections::Vector,
  crypto::{
    AEAD_NONCE_LEN, AEAD_TAG_LEN, Aead as _, Aes256GcmGlobal, Hash as _, Sha256Global,
    gen_aead_nonce,
  },
  misc::{LeaseMut as _, SensitiveBytes, mlock_slice, munlock_slice},
  rng::{ChaCha20, CryptoSeedableRng as _, Rng as _},
  secret::{ProtectedChunk, SecretData},
};
use alloc::string::String;
use core::{
  fmt::{Debug, Formatter},
  ops::Deref,
  range::Range,
};

// In theory a larger context is more secure but at the same time a binary with several secrets
// could cause trouble in restricted hardware.
const CTX_LEN: usize = 256;
const SECRET_LEN: usize = 32;

/// Encrypted chunk
pub struct EncryptedChunk {
  aead: ProtectedChunk,
  context: ProtectedChunk,
  salt: [u8; SECRET_LEN],
}

impl EncryptedChunk {
  /// New instance
  #[inline]
  pub fn new(data: &mut [u8]) -> crate::Result<Self> {
    let mut context = ProtectedChunk::zeroed(CTX_LEN);
    let mut data_wrapper = SensitiveBytes::new(data);
    let mut rng = ChaCha20::from_std_random()?;
    let mut salt = [0; SECRET_LEN];
    rng.fill_slice(&mut context);
    rng.fill_slice(&mut salt);
    let nonce = gen_aead_nonce(&mut rng);
    let secret = gen_secret_key(&context, &salt);
    let tag = Aes256GcmGlobal::encrypt_parts(&[], nonce, &mut data_wrapper, &secret)?;
    let aead = gen_aead(&data_wrapper, nonce, tag);
    mlock_slice(&mut context)?;
    Ok(Self { aead, context, salt })
  }

  /// Data
  #[inline]
  pub(crate) fn data(&self) -> crate::Result<(Vector<u8>, Range<usize>)> {
    let mut buffer = Vector::new();
    buffer.extend_from_copyable_slice(&self.aead)?;
    let (_, range) = Aes256GcmGlobal::decrypt_in_place(
      &[],
      buffer.lease_mut(),
      &gen_secret_key(&self.context, &self.salt),
    )?;
    Ok((buffer, range))
  }
}

impl Debug for EncryptedChunk {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Secret").finish()
  }
}

impl Default for EncryptedChunk {
  #[inline]
  fn default() -> Self {
    Self {
      context: ProtectedChunk::zeroed(0),
      aead: ProtectedChunk::zeroed(0),
      salt: [0; SECRET_LEN],
    }
  }
}

impl Drop for EncryptedChunk {
  #[inline]
  fn drop(&mut self) {
    let _rslt = munlock_slice(&mut self.context);
    drop(SensitiveBytes::new(self.salt));
  }
}

#[inline]
#[rustfmt::skip]
fn gen_aead(
  encrypted: &SensitiveBytes<&mut [u8]>,
  nonce: [u8; AEAD_NONCE_LEN],
  tag: [u8; AEAD_TAG_LEN]
) -> ProtectedChunk {
  let all_len = nonce.len().wrapping_add(encrypted.len()).wrapping_add(tag.len());
  let mut aead = ProtectedChunk::zeroed(all_len);
  let rest = if let Some((enc_nonce, rest)) = aead.split_first_chunk_mut::<AEAD_NONCE_LEN>() {
    enc_nonce.copy_from_slice(&nonce);
    rest
  } else {
    // Unreachable
    &mut []
  };
  if let Some((content, enc_tag)) = rest.split_last_chunk_mut::<AEAD_TAG_LEN>() {
    content.copy_from_slice(encrypted);
    enc_tag.copy_from_slice(&tag);
  }
  aead
}

fn gen_secret_key(context: &[u8], salt: &[u8; SECRET_LEN]) -> SensitiveBytes<[u8; SECRET_LEN]> {
  SensitiveBytes::new(Sha256Global::digest([context, &salt[..]]))
}

/// An [`EncryptedChunk`] that represents a fixed-size array of secret bytes.
#[derive(Debug)]
#[repr(transparent)]
pub struct EncryptedChunkArray<const N: usize>(EncryptedChunk);
impl<const N: usize> SecretData for EncryptedChunkArray<N> {
  type ReprSrc<'this> = EncryptedChunkArrayReprSrc<N>;

  #[inline]
  fn new(data: &mut <Self::ReprSrc<'_> as Deref>::Target) -> crate::Result<Self> {
    Ok(Self(EncryptedChunk::new(data)?))
  }

  #[inline]
  fn repr_src(&self) -> crate::Result<Self::ReprSrc<'_>> {
    let (buffer, range) = self.0.data()?;
    Ok(EncryptedChunkArrayReprSrc(SensitiveBytes::new(buffer), range))
  }
}
/// Temporary plaintext guard for [`EncryptedChunkArray`].
#[derive(Debug)]
pub struct EncryptedChunkArrayReprSrc<const N: usize>(SensitiveBytes<Vector<u8>>, Range<usize>);
impl<const N: usize> Deref for EncryptedChunkArrayReprSrc<N> {
  type Target = [u8; N];

  #[inline]
  fn deref(&self) -> &Self::Target {
    let slice = self.0.get(self.1).unwrap_or_default();
    // SAFETY: The bytes in this instance have the same layout of the bytes used when constructing
    unsafe { slice.as_array().unwrap_unchecked() }
  }
}

/// An [`EncryptedChunk`] that represents a dynamically-sized slice of secret bytes.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct EncryptedChunkSlice(EncryptedChunk);
impl SecretData for EncryptedChunkSlice {
  type ReprSrc<'this> = EncryptedChunkSliceReprSrc;

  #[inline]
  fn new(data: &mut <Self::ReprSrc<'_> as Deref>::Target) -> crate::Result<Self> {
    Ok(Self(EncryptedChunk::new(data)?))
  }

  #[inline]
  fn repr_src(&self) -> crate::Result<Self::ReprSrc<'_>> {
    let (buffer, range) = self.0.data()?;
    Ok(EncryptedChunkSliceReprSrc(SensitiveBytes::new(buffer), range))
  }
}
/// Temporary plaintext guard for [`EncryptedChunkSlice`].
#[derive(Debug)]
pub struct EncryptedChunkSliceReprSrc(SensitiveBytes<Vector<u8>>, Range<usize>);
impl Deref for EncryptedChunkSliceReprSrc {
  type Target = [u8];

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.0.get(self.1).unwrap_or_default()
  }
}

/// An [`EncryptedChunk`] that represents a valid UTF-8 secret string.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct EncryptedChunkStr(EncryptedChunk);
impl SecretData for EncryptedChunkStr {
  type ReprSrc<'this> = EncryptedChunkStrReprSrc;

  #[inline]
  fn new(data: &mut <Self::ReprSrc<'_> as Deref>::Target) -> crate::Result<Self> {
    // SAFETY: `data` is zeroed by `EncryptedChunk` and since are zeros are ASCII, the conversion
    //         is safe.
    let bytes = unsafe { data.as_bytes_mut() };
    Ok(Self(EncryptedChunk::new(bytes)?))
  }

  #[inline]
  fn repr_src(&self) -> crate::Result<Self::ReprSrc<'_>> {
    let (buffer, range) = self.0.data()?;
    Ok(EncryptedChunkStrReprSrc(SensitiveBytes::new(buffer), range))
  }
}
/// Temporary plaintext guard for [`EncryptedChunkStr`].
#[derive(Debug)]
pub struct EncryptedChunkStrReprSrc(SensitiveBytes<Vector<u8>>, Range<usize>);
impl Deref for EncryptedChunkStrReprSrc {
  type Target = str;

  #[inline]
  fn deref(&self) -> &Self::Target {
    let bytes = self.0.get(self.1).unwrap_or_default();
    // SAFETY: The bytes of this instance have the same layout of the bytes used when constructing
    unsafe { str::from_utf8_unchecked(bytes) }
  }
}
impl TryFrom<String> for EncryptedChunkStr {
  type Error = crate::Error;

  #[inline]
  fn try_from(mut value: String) -> crate::Result<Self> {
    Self::new(value.as_mut_str())
  }
}
