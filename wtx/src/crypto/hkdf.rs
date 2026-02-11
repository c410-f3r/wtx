/// HMAC-based Key Derivation Function
pub trait Hkdf: Sized {
  /// Hash
  type Hash;

  /// The HKDF-Extract operation from the RFC-5869
  fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Hash, Self);

  /// Creates a new instance from an already cryptographically strong pseudorandom key.
  fn from_prk(prk: &[u8]) -> crate::Result<Self>;

  /// Performs a one-shot HMAC
  fn compute<'data>(
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<Self::Hash>;

  /// The HKDF-Expand operation from the RFC-5869
  fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()>;
}

impl Hkdf for () {
  type Hash = [u8; 0];

  #[inline]
  fn extract(_: Option<&[u8]>, _: &[u8]) -> (Self::Hash, Self) {
    ([0; 0], ())
  }

  #[inline]
  fn from_prk(_: &[u8]) -> crate::Result<Self> {
    Ok(())
  }

  #[inline]
  fn compute<'data>(
    _: impl IntoIterator<Item = &'data [u8]>,
    _: &[u8],
  ) -> crate::Result<Self::Hash> {
    Ok([0; 0])
  }

  #[inline]
  fn expand(&self, _: &[u8], _: &mut [u8]) -> crate::Result<()> {
    Ok(())
  }
}

#[cfg(feature = "aws-lc-rs")]
mod aws_lc_rs {
  use crate::{
    crypto::{CryptoError, Hkdf, Sha256HkdfAwsLcRs, Sha384HkdfAwsLcRs},
    misc::unlikely_elem,
  };
  use aws_lc_rs::{
    hkdf::{self, HKDF_SHA256, HKDF_SHA384, Prk},
    hmac::{self, Context, HMAC_SHA256, HMAC_SHA384, Key},
  };

  impl Hkdf for Sha256HkdfAwsLcRs {
    type Hash = [u8; 32];

    #[inline]
    fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Hash, Self) {
      let (lhs, rhs) = local_extract::<32>(HKDF_SHA256, HMAC_SHA256, salt, ikm);
      (lhs, Self::new(rhs))
    }

    #[inline]
    fn from_prk(prk: &[u8]) -> crate::Result<Self> {
      Ok(Self::new(Prk::new_less_safe(HKDF_SHA256, prk)))
    }

    #[inline]
    fn compute<'data>(
      data: impl IntoIterator<Item = &'data [u8]>,
      key: &[u8],
    ) -> crate::Result<Self::Hash> {
      local_compute(HMAC_SHA256, data, key)
    }

    #[inline]
    fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
      local_expand(HKDF_SHA256, info, okm, &self.value)
    }
  }

  impl Hkdf for Sha384HkdfAwsLcRs {
    type Hash = [u8; 48];

    #[inline]
    fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Hash, Self) {
      let (lhs, rhs) = local_extract::<48>(HKDF_SHA384, HMAC_SHA384, salt, ikm);
      (lhs, Self::new(rhs))
    }

    #[inline]
    fn from_prk(prk: &[u8]) -> crate::Result<Self> {
      Ok(Self::new(Prk::new_less_safe(HKDF_SHA384, prk)))
    }

    #[inline]
    fn compute<'data>(
      data: impl IntoIterator<Item = &'data [u8]>,
      key: &[u8],
    ) -> crate::Result<Self::Hash> {
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
  ) -> ([u8; N], Prk) {
    let salt_bytes = match salt {
      Some(el) => el,
      None => &[0u8; N],
    };
    let key = Key::new(algorithm_hmac, salt_bytes);
    let tag = hmac::sign(&key, ikm);
    let prk_bytes = tag.as_ref();
    let array = if let Ok(elem) = tag.as_ref().try_into() { elem } else { unlikely_elem([0; N]) };
    let prk = Prk::new_less_safe(algorithm_hkdf, prk_bytes);
    (array, prk)
  }

  #[inline]
  fn local_compute<'data, const N: usize>(
    algorithm: hmac::Algorithm,
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<[u8; N]> {
    let key = Key::new(algorithm, key);
    let mut ctx = Context::with_key(&key);
    for chunk in data {
      ctx.update(chunk);
    }
    Ok(ctx.sign().as_ref().try_into()?)
  }

  #[inline]
  fn local_expand(
    algorithm: hkdf::Algorithm,
    info: &[u8],
    okm: &mut [u8],
    value: &Prk,
  ) -> crate::Result<()> {
    let mut fun = || value.expand(&[info], algorithm).ok()?.fill(okm).ok();
    fun().ok_or(CryptoError::HkdfExpandError)?;
    Ok(())
  }
}

#[cfg(feature = "hkdf")]
mod hkdf {
  use crate::crypto::{CryptoError, Hkdf};
  use crypto_common::{KeyInit, Output};
  use digest::OutputSizeUser;
  use hmac::{EagerHash, Mac, SimpleHmac};

  impl<H> Hkdf for hkdf::Hkdf<H>
  where
    H: EagerHash,
    H::Core: OutputSizeUser<OutputSize = H::OutputSize>,
  {
    type Hash = Output<H>;

    #[inline]
    fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (Self::Hash, Self) {
      let (lhs, rhs) = hkdf::Hkdf::<H>::extract(salt, ikm);
      (lhs, rhs)
    }

    #[inline]
    fn from_prk(prk: &[u8]) -> crate::Result<Self> {
      Ok(hkdf::Hkdf::from_prk(prk).map_err(|_err| CryptoError::HkdfFromPrkError)?)
    }

    #[inline]
    fn compute<'data>(
      data: impl IntoIterator<Item = &'data [u8]>,
      key: &[u8],
    ) -> crate::Result<Self::Hash> {
      let mut hmac = SimpleHmac::<H>::new_from_slice(key)?;
      for elem in data {
        Mac::update(&mut hmac, elem);
      }
      Ok(hmac.finalize().into_bytes())
    }

    #[inline]
    fn expand(&self, info: &[u8], okm: &mut [u8]) -> crate::Result<()> {
      hkdf::Hkdf::<H>::expand(self, info, okm).map_err(|_err| CryptoError::HkdfExpandError)?;
      Ok(())
    }
  }
}
