#[cfg(any(feature = "crypto-aws-lc-rs", feature = "crypto-ring"))]
macro_rules! common_aead_functions {
  () => {
    #[inline]
    fn local_decrypt<'data, const S: usize>(
      algorithm: &'static Algorithm,
      associated_data: &[u8],
      data: &'data mut [u8],
      error: CryptoError,
      nonce: [u8; AEAD_NONCE_LEN],
      secret: &[u8; S],
    ) -> crate::Result<&'data mut [u8]> {
      let bytes = LessSafeKey::new(UnboundKey::new(algorithm, secret).map_err(|_err| error)?)
        .open_in_place(Nonce::assume_unique_for_key(nonce), Aad::from(associated_data), data)
        .map_err(|_err| error)?;
      Ok(bytes)
    }

    #[inline]
    fn local_encrypt_vectored_data<const S: usize>(
      algorithm: &'static Algorithm,
      associated_data: &[u8],
      error: CryptoError,
      nonce: [u8; AEAD_NONCE_LEN],
      plaintext: &mut [u8],
      secret: &[u8; S],
    ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
      let tag = LessSafeKey::new(UnboundKey::new(algorithm, secret).map_err(|_err| error)?)
        .seal_in_place_separate_tag(
          Nonce::assume_unique_for_key(nonce),
          Aad::from(associated_data),
          plaintext,
        )
        .map_err(|_err| error)?
        .as_ref()
        .try_into()?;
      Ok(tag)
    }
  };
}

#[cfg(any(feature = "crypto-aws-lc-rs", feature = "crypto-ring"))]
macro_rules! common_hkdf_functions {
  () => {
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
  };
}

macro_rules! _create_wrappers {
  ($(
    $(#[$meta:meta])*
    $name:ident<$($param:ident $(: $bound:path)?)?>($($ty:ty)?)
  ),* $(,)?) => {
    $(
      /// A wrapper used to generalize third-party dependencies.
      $(#[$meta])*
      #[allow(missing_copy_implementations, reason = "too many structures to control")]
      pub struct $name<$($param $(: $bound)?)?>($(pub(crate) $ty)?);

      impl<$($param $(: $bound)?)?> $name<$($param)?> {
        /// New instance
        #[inline]
        pub const fn new($(value: $ty)?) -> Self {
          Self(
            $({
              let _expander: Option<$ty> = None;
              value
            })?
          )
        }
      }

      impl core::fmt::Debug for $name {
        #[inline]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
          f.write_str(stringify!($name))
        }
      }
    )*
  };
}
