#[cfg(any(feature = "crypto-aws-lc-rs", feature = "crypto-ring"))]
macro_rules! common_aead_functions {
  () => {
    #[inline]
    fn local_decrypt<'encrypted, const S: usize>(
      algorithm: &'static Algorithm,
      associated_data: &[u8],
      encrypted_data: &'encrypted mut [u8],
      error: CryptoError,
      secret: &[u8; S],
    ) -> crate::Result<&'encrypted mut [u8]> {
      let (nonce, content) = split_nonce_content(encrypted_data, error)?;
      let bytes = LessSafeKey::new(UnboundKey::new(algorithm, secret).map_err(|_| error)?)
        .open_in_place(Nonce::assume_unique_for_key(nonce), Aad::from(associated_data), content)
        .map_err(|_| error)?;
      Ok(bytes)
    }

    #[inline]
    fn local_encrypt_vectored_data<RNG, const S: usize>(
      algorithm: &'static Algorithm,
      associated_data: &[u8],
      error: CryptoError,
      nonce: [&mut u8; NONCE_LEN],
      plaintext: &mut [u8],
      rng: &mut RNG,
      secret: &[u8; S],
      tag: [&mut u8; TAG_LEN],
    ) -> crate::Result<()>
    where
      RNG: CryptoRng,
    {
      let local_tag = LessSafeKey::new(UnboundKey::new(algorithm, secret).map_err(|_| error)?)
        .seal_in_place_separate_tag(
          Nonce::assume_unique_for_key(generate_nonce(nonce, rng)),
          Aad::from(associated_data),
          plaintext,
        )
        .map_err(|_| error)?
        .as_ref()
        .try_into()?;
      write_tag(local_tag, tag);
      Ok(())
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
      #[derive(Debug)]
      $(#[$meta])*
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
    )*
  };
}
