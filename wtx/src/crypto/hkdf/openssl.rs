#![expect(
  clippy::unwrap_used,
  reason = "it is not worth changing the signature because of one backend"
)]

use crate::crypto::{Hkdf, HkdfSha256Openssl, HkdfSha384Openssl};
use openssl::{
  hash::MessageDigest,
  md::{Md, MdRef},
  pkey::{Id, PKey},
  pkey_ctx::{HkdfMode, PkeyCtx},
  sign::Signer,
};

impl Hkdf for HkdfSha256Openssl {
  type Digest = [u8; 32];

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let prk = local_extract(ikm, Md::sha256(), salt).unwrap();
    (prk, Self(prk))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(prk.try_into()?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    local_compute(data, key, MessageDigest::sha256())
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    local_expand(info, Md::sha256(), okm, &self.0)
  }
}

impl Hkdf for HkdfSha384Openssl {
  type Digest = [u8; 48];

  #[inline]
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Digest, Self) {
    let prk = local_extract(ikm, Md::sha384(), salt).unwrap();
    (prk, Self(prk))
  }

  #[inline]
  fn from_prk(prk: &[u8]) -> crate::Result<Self> {
    Ok(Self::new(prk.try_into()?))
  }

  #[inline]
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Digest> {
    local_compute(data, key, MessageDigest::sha384())
  }

  #[inline]
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
    local_expand(info, Md::sha384(), okm, &self.0)
  }
}

#[inline]
fn local_compute<'data, const N: usize>(
  data: impl IntoIterator<Item = &'data [u8]>,
  key: &[u8],
  md: MessageDigest,
) -> crate::Result<[u8; N]> {
  let pkey = PKey::hmac(key)?;
  let mut signer = Signer::new(md, &pkey)?;
  for chunk in data {
    signer.update(chunk)?;
  }
  let mut array = [0u8; N];
  let _ = signer.sign(&mut array)?;
  Ok(array)
}

#[inline]
fn local_expand(info: &[u8], md: &MdRef, okm: &mut [u8], prk: &[u8]) -> crate::Result<()> {
  let mut ctx = PkeyCtx::new_id(Id::HKDF)?;
  ctx.derive_init()?;
  ctx.set_hkdf_mode(HkdfMode::EXPAND_ONLY)?;
  ctx.set_hkdf_md(md)?;
  ctx.set_hkdf_key(prk)?;
  ctx.add_hkdf_info(info)?;
  let _ = ctx.derive(Some(okm))?;
  Ok(())
}

#[inline]
fn local_extract<const N: usize>(
  ikm: &[u8],
  md: &MdRef,
  salt: Option<&[u8]>,
) -> crate::Result<[u8; N]> {
  let mut ctx = PkeyCtx::new_id(Id::HKDF)?;
  ctx.derive_init()?;
  ctx.set_hkdf_mode(HkdfMode::EXTRACT_ONLY)?;
  ctx.set_hkdf_md(md)?;
  ctx.set_hkdf_key(ikm)?;
  if let Some(el) = salt {
    ctx.set_hkdf_salt(el)?;
  }
  let mut array = [0u8; N];
  let _ = ctx.derive(Some(&mut array))?;
  Ok(array)
}
