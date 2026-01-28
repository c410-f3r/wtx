use crate::{collection::ArrayVectorU8, crypto::MAX_HASH_LEN};

/// HMAC-based Key Derivation Function
pub trait Hkdf: Sized {
  /// The HKDF-Extract operation from the RFC-5869
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self);

  /// Creates a new instance from an already cryptographically strong pseudorandom key.
  fn from_prk(prk: &[u8]) -> crate::Result<Self>;

  /// Performs a one-shot HMAC
  fn compute<'data>(
    data: impl Iterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>>;

  /// The HKDF-Expand operation from the RFC-5869
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()>;
}

impl Hkdf for () {
  #[inline]
  fn extract(_: Option<&[u8]>, _: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
    (ArrayVectorU8::new(), ())
  }

  #[inline]
  fn from_prk(_: &[u8]) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  fn compute<'data>(
    _: impl Iterator<Item = &'data [u8]>,
    _: &[u8],
  ) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
    Ok(ArrayVectorU8::new())
  }

  #[inline]
  fn expand(&self, _: &[u8], _: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "aws-lc-rs")]
mod aws_lc_rs {
  use crate::{
    collection::ArrayVectorU8,
    crypto::{Hkdf, MAX_HASH_LEN, Sha256HkdfAwsLcRs, Sha384HkdfAwsLcRs},
  };
  use aws_lc_rs::{
    hkdf::{self, HKDF_SHA256, HKDF_SHA384, Prk},
    hmac::{self, Context, HMAC_SHA256, HMAC_SHA384, Key},
  };

  impl Hkdf for Sha256HkdfAwsLcRs {
    #[inline]
    fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
      let (lhs, rhs) = local_extract::<32>(HKDF_SHA256, HMAC_SHA256, salt, ikm);
      (lhs, Self::new(rhs))
    }

    #[inline]
    fn from_prk(prk: &[u8]) -> crate::Result<Self> {
      Ok(Self::new(Prk::new_less_safe(HKDF_SHA256, prk)))
    }

    #[inline]
    fn compute<'data>(
      data: impl Iterator<Item = &'data [u8]>,
      key: &[u8],
    ) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
      local_compute(HMAC_SHA256, data, key)
    }

    #[inline]
    fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
      local_expand(HKDF_SHA256, info, okm, &self.value)
    }
  }

  impl Hkdf for Sha384HkdfAwsLcRs {
    #[inline]
    fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
      let (lhs, rhs) = local_extract::<48>(HKDF_SHA384, HMAC_SHA384, salt, ikm);
      (lhs, Self::new(rhs))
    }

    #[inline]
    fn from_prk(prk: &[u8]) -> crate::Result<Self> {
      Ok(Self::new(Prk::new_less_safe(HKDF_SHA384, prk)))
    }

    #[inline]
    fn compute<'data>(
      data: impl Iterator<Item = &'data [u8]>,
      key: &[u8],
    ) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
      local_compute(HMAC_SHA384, data, key)
    }

    #[inline]
    fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
      local_expand(HKDF_SHA384, info, okm, &self.value)
    }
  }

  #[inline]
  fn local_extract<const N: usize>(
    algorithm_hkdf: hkdf::Algorithm,
    algorithm_hmac: hmac::Algorithm,
    salt: Option<&[u8]>,
    ikm: &[u8],
  ) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Prk) {
    let salt_bytes = match salt {
      Some(el) => el,
      None => &[0u8; N],
    };
    let key = Key::new(algorithm_hmac, salt_bytes);
    let tag = hmac::sign(&key, ikm);
    let prk_bytes = tag.as_ref();
    let mut array = ArrayVectorU8::new();
    let _rslt = array.extend_from_copyable_slice(prk_bytes);
    let prk = Prk::new_less_safe(algorithm_hkdf, prk_bytes);
    (array, prk)
  }

  #[inline]
  fn local_compute<'data>(
    algorithm: hmac::Algorithm,
    data: impl Iterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
    let key = Key::new(algorithm, key);
    let mut ctx = Context::with_key(&key);
    for chunk in data {
      ctx.update(chunk);
    }
    let tag = ctx.sign();
    let mut rslt = ArrayVectorU8::new();
    rslt.extend_from_copyable_slice(tag.as_ref())?;
    Ok(rslt)
  }

  #[inline]
  fn local_expand(
    algorithm: hkdf::Algorithm,
    info: &[u8],
    okm: &mut [u8],
    value: &hkdf::Prk,
  ) -> crate::Result<()> {
    let mut fun = || value.expand(&[info], algorithm).ok()?.fill(okm).ok();
    fun().ok_or(crate::crypto::CryptoError::HkdfExpandError)?;
    Ok(())
  }
}

#[cfg(feature = "hkdf")]
mod hkdf {
  use crate::{
    collection::ArrayVectorU8,
    crypto::{CryptoError, Hkdf, MAX_HASH_LEN},
  };
  use hkdf::hmac::{EagerHash, Mac, SimpleHmac, digest::KeyInit};

  impl<H> Hkdf for hkdf::Hkdf<H>
  where
    H: EagerHash,
  {
    #[inline]
    fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (ArrayVectorU8<u8, MAX_HASH_LEN>, Self) {
      let (lhs, rhs) = hkdf::Hkdf::extract(salt, ikm);
      let mut array = ArrayVectorU8::new();
      let _rslt = array.extend_from_copyable_slice(&lhs);
      (array, rhs)
    }

    #[inline]
    fn from_prk(prk: &[u8]) -> crate::Result<Self> {
      Ok(hkdf::Hkdf::from_prk(prk).map_err(|_err| CryptoError::HkdfFromPrkError)?)
    }

    #[inline]
    fn compute<'data>(
      data: impl Iterator<Item = &'data [u8]>,
      key: &[u8],
    ) -> crate::Result<ArrayVectorU8<u8, MAX_HASH_LEN>> {
      let mut hmac = SimpleHmac::<H>::new_from_slice(key)?;
      for elem in data {
        Mac::update(&mut hmac, elem);
      }
      let mut rslt = ArrayVectorU8::new();
      rslt.extend_from_copyable_slice(&hmac.finalize().into_bytes())?;
      Ok(rslt)
    }

    #[inline]
    fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
      hkdf::Hkdf::<H>::expand(self, info, okm)
        .map_err(|_err| crate::crypto::CryptoError::HkdfExpandError)?;
      Ok(())
    }
  }
}
