use crate::{
  collections::TryExtend,
  crypto::{
    AEAD_NONCE_LEN, AEAD_TAG_LEN, Aead as _, Aes256GcmGlobal, Hash as _, Sha256Global,
    gen_aead_nonce,
  },
  misc::{LeaseMut, SecretContext, SensitiveBytes, secret::protected::Protected},
  rng::CryptoRng,
  sync::Arc,
};
use core::{
  fmt::{Debug, Formatter},
  range::Range,
};

const SECRET_LEN: usize = 32;

pub(crate) struct Encrypted {
  protected: Protected,
  salt: [u8; SECRET_LEN],
  secret_context: SecretContext,
}

impl Encrypted {
  #[inline]
  pub(crate) fn new<RNG>(
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
    let tag = Aes256GcmGlobal::encrypt_parts(
      &[],
      nonce,
      &mut data_wrapper,
      &gen_secret_key(&salt, &secret_context),
    )?;
    Ok(Self { protected: gen_protected(&data_wrapper, nonce, tag), salt, secret_context })
  }

  #[inline]
  pub(crate) fn peek<B>(&self, buffer: &mut B) -> crate::Result<(usize, Range<usize>)>
  where
    for<'any> B: LeaseMut<[u8]> + TryExtend<&'any [u8]>,
  {
    let idx = buffer.lease().len();
    buffer.try_extend(&self.protected)?;
    let (_, range) = Aes256GcmGlobal::decrypt_in_place(
      &[],
      buffer.lease_mut().get_mut(idx..).unwrap_or_default(),
      &gen_secret_key(&self.salt, &self.secret_context),
    )?;
    Ok((idx, range))
  }
}

impl Debug for Encrypted {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Secret").finish()
  }
}

impl Default for Encrypted {
  #[inline]
  fn default() -> Self {
    Self {
      protected: Protected::zeroed(0),
      salt: [0; SECRET_LEN],
      secret_context: SecretContext { _inner: Arc::new(Protected::zeroed(0)) },
    }
  }
}

#[inline]
#[rustfmt::skip]
fn gen_protected(
  encrypted: &SensitiveBytes<&mut [u8]>,
  nonce: [u8; AEAD_NONCE_LEN],
  tag: [u8; AEAD_TAG_LEN]
) -> Protected {
  let all_len = nonce.len().wrapping_add(encrypted.len()).wrapping_add(tag.len());
  let mut protected = Protected::zeroed(all_len);
  let rest = if let Some((enc_nonce, rest)) = protected.split_first_chunk_mut::<AEAD_NONCE_LEN>() {
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
  protected
}

fn gen_secret_key(salt: &[u8; SECRET_LEN], secret_context: &SecretContext) -> [u8; SECRET_LEN] {
  Sha256Global::digest([&salt[..], &secret_context._inner])
}
