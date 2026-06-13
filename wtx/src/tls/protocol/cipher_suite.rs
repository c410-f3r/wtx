use crate::{
  codec::{Decode, Encode},
  crypto::{
    Aead, Aes128GcmGlobal, Aes256GcmGlobal, Chacha20Poly1305Global, Hash, Hkdf, HkdfSha256Global,
    HkdfSha384Global, Hmac, HmacSha256Global, HmacSha384Global, Sha256HashGlobal, Sha384HashGlobal,
  },
  misc::Either,
  tls::{
    TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper,
    tls_hash::TlsHash, tls_hkdf::TlsHkdf, tls_hmac::TlsHmac,
  },
};

create_enum! {
  /// Refers a concrete cipher suite implementation.
  #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
  pub enum CipherSuite<u16> {
    /// Aes128GcmSha256
    #[default]
    Aes128GcmSha256 = (0x1301),
    /// Aes256GcmSha384
    Aes256GcmSha384 = (0x1302),
    /// Chacha20Poly1305Sha256
    Chacha20Poly1305Sha256 = (0x1303),
  }
}

impl CipherSuite {
  #[inline]
  pub(crate) fn aes_decrypt<'data>(
    self,
    associated_data: &[u8],
    data: &'data mut [u8],
    nonce: [u8; 12],
    secret: &[u8],
  ) -> crate::Result<&'data mut [u8]> {
    match self {
      CipherSuite::Aes128GcmSha256 => {
        Aes128GcmGlobal::decrypt_parts(associated_data, data, nonce, secret.try_into()?)
      }
      CipherSuite::Aes256GcmSha384 => {
        Aes256GcmGlobal::decrypt_parts(associated_data, data, nonce, secret.try_into()?)
      }
      CipherSuite::Chacha20Poly1305Sha256 => {
        Chacha20Poly1305Global::decrypt_parts(associated_data, data, nonce, secret.try_into()?)
      }
    }
  }

  #[inline]
  pub(crate) fn cipher_key_len(self) -> u8 {
    match self {
      CipherSuite::Aes128GcmSha256 => 16,
      CipherSuite::Aes256GcmSha384 => 32,
      CipherSuite::Chacha20Poly1305Sha256 => 32,
    }
  }

  #[inline]
  pub(crate) fn hash_digest<'data>(self, data: impl IntoIterator<Item = &'data [u8]>) -> TlsHash {
    match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        Either::Left(Sha256HashGlobal::digest(data))
      }
      CipherSuite::Aes256GcmSha384 => Either::Right(Sha384HashGlobal::digest(data)),
    }
  }

  #[inline]
  pub(crate) fn hash_len(self) -> u8 {
    match self {
      CipherSuite::Aes128GcmSha256 => 32,
      CipherSuite::Aes256GcmSha384 => 48,
      CipherSuite::Chacha20Poly1305Sha256 => 32,
    }
  }

  #[inline]
  pub(crate) fn hkdf_compute<'data>(
    self,
    data: impl IntoIterator<Item = &'data [u8]>,
    key: &[u8],
  ) -> crate::Result<TlsHash> {
    Ok(match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        Either::Left(HkdfSha256Global::compute(data, key)?)
      }
      CipherSuite::Aes256GcmSha384 => Either::Right(HkdfSha384Global::compute(data, key)?),
    })
  }

  #[inline]
  pub(crate) fn hkdf_extract(self, salt: Option<&[u8]>, ikm: &[u8]) -> TlsHkdf {
    match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        Either::Left(HkdfSha256Global::extract(salt, ikm).1)
      }
      CipherSuite::Aes256GcmSha384 => Either::Right(HkdfSha384Global::extract(salt, ikm).1),
    }
  }

  #[inline]
  pub(crate) fn hkdf_from_prk(self, prk: &[u8]) -> crate::Result<TlsHkdf> {
    Ok(match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        Either::Left(HkdfSha256Global::from_prk(prk)?)
      }
      CipherSuite::Aes256GcmSha384 => Either::Right(HkdfSha384Global::from_prk(prk)?),
    })
  }

  #[inline]
  pub(crate) fn hmac_from_key(self, key: &[u8]) -> crate::Result<TlsHmac> {
    Ok(match self {
      CipherSuite::Aes128GcmSha256 => Either::Left(HmacSha256Global::from_key(key)?),
      CipherSuite::Aes256GcmSha384 => Either::Right(HmacSha384Global::from_key(key)?),
      CipherSuite::Chacha20Poly1305Sha256 => Either::Left(HmacSha256Global::from_key(key)?),
    })
  }
}

impl<'de> Decode<'de, De> for CipherSuite {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let [a, b, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidMaxFragmentLength.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self::try_from(u16::from_be_bytes([*a, *b]))?)
  }
}

impl Encode<De> for CipherSuite {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}
