use crate::{
  codec::{Decode, Encode},
  collections::ArrayVectorCopy,
  crypto::{
    AEAD_TAG_LEN, Aead as _, Aes128GcmGlobal, Aes256GcmGlobal, Chacha20Poly1305Global, Hash as _,
    Hkdf as _, HkdfSha256Global, HkdfSha384Global, Hmac as _, HmacSha256Global, HmacSha384Global,
    MAX_HASH_LEN, Sha256HashGlobal, Sha384HashGlobal,
  },
  tls::{
    TlsError,
    de::De,
    tls_decode_wrapper::TlsDecodeWrapper,
    tls_encode_wrapper::TlsEncodeWrapper,
    tls_hash::{TlsDigest, TlsHash},
    tls_hkdf::TlsHkdf,
    tls_hmac::TlsHmac,
  },
};

/// Refers a concrete cipher suite implementation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CipherSuite {
  /// `Aes128GcmSha256`
  #[default]
  Aes128GcmSha256 = 0x1301,
  /// `Aes256GcmSha384`
  Aes256GcmSha384 = 0x1302,
  /// `Chacha20Poly1305Sha256`
  Chacha20Poly1305Sha256 = 0x1303,
}

impl CipherSuite {
  pub(crate) const ALL: [Self; 3] =
    [Self::Aes128GcmSha256, Self::Aes256GcmSha384, Self::Chacha20Poly1305Sha256];

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
  pub(crate) fn aes_encrypt(
    self,
    associated_data: &[u8],
    data: &mut [u8],
    nonce: [u8; 12],
    secret: &[u8],
  ) -> crate::Result<[u8; AEAD_TAG_LEN]> {
    match self {
      CipherSuite::Aes128GcmSha256 => {
        Aes128GcmGlobal::encrypt_parts(associated_data, nonce, data, secret.try_into()?)
      }
      CipherSuite::Aes256GcmSha384 => {
        Aes256GcmGlobal::encrypt_parts(associated_data, nonce, data, secret.try_into()?)
      }
      CipherSuite::Chacha20Poly1305Sha256 => {
        Chacha20Poly1305Global::encrypt_parts(associated_data, nonce, data, secret.try_into()?)
      }
    }
  }

  #[inline]
  pub(crate) fn cipher_key_len(self) -> u8 {
    match self {
      CipherSuite::Aes128GcmSha256 => 16,
      CipherSuite::Aes256GcmSha384 | CipherSuite::Chacha20Poly1305Sha256 => 32,
    }
  }

  #[inline]
  pub(crate) fn hash_digest<'data>(self, data: impl IntoIterator<Item = &'data [u8]>) -> TlsDigest {
    match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        TlsDigest::Sha256(Sha256HashGlobal::digest(data))
      }
      CipherSuite::Aes256GcmSha384 => TlsDigest::Sha384(Sha384HashGlobal::digest(data)),
    }
  }

  #[inline]
  pub(crate) fn hash_len(self) -> u8 {
    match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => 32,
      CipherSuite::Aes256GcmSha384 => 48,
    }
  }

  #[inline]
  pub(crate) fn hash_new(self) -> TlsHash {
    match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        TlsHash::Sha256(Sha256HashGlobal::new())
      }
      CipherSuite::Aes256GcmSha384 => TlsHash::Sha384(Sha384HashGlobal::new()),
    }
  }

  #[inline]
  pub(crate) fn hkdf_extract(self, salt: Option<&[u8]>, ikm: &[u8]) -> TlsHkdf {
    match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        TlsHkdf::Sha256(HkdfSha256Global::extract(salt, ikm).1)
      }
      CipherSuite::Aes256GcmSha384 => TlsHkdf::Sha384(HkdfSha384Global::extract(salt, ikm).1),
    }
  }

  #[inline]
  pub(crate) fn hkdf_from_prk(self, prk: &[u8]) -> crate::Result<TlsHkdf> {
    Ok(match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        TlsHkdf::Sha256(HkdfSha256Global::from_prk(prk)?)
      }
      CipherSuite::Aes256GcmSha384 => TlsHkdf::Sha384(HkdfSha384Global::from_prk(prk)?),
    })
  }

  #[inline]
  pub(crate) fn hmac_from_key(self, key: &[u8]) -> crate::Result<TlsHmac> {
    Ok(match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        TlsHmac::Sha256(HmacSha256Global::from_key(key)?)
      }
      CipherSuite::Aes256GcmSha384 => TlsHmac::Sha384(HmacSha384Global::from_key(key)?),
    })
  }

  #[inline]
  pub(crate) fn zeroed_hash(self) -> ArrayVectorCopy<u8, MAX_HASH_LEN> {
    match self {
      CipherSuite::Aes128GcmSha256 | CipherSuite::Chacha20Poly1305Sha256 => {
        ArrayVectorCopy::from_array([
          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
          0, 0,
        ])
      }
      CipherSuite::Aes256GcmSha384 => ArrayVectorCopy::from_array([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      ]),
    }
  }
}

impl<'de> Decode<'de, De> for CipherSuite {
  #[inline]
  fn decode(dw: &mut TlsDecodeWrapper<'de>) -> crate::Result<Self> {
    let [b0, b1, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidCipherSuite.into());
    };
    let element = Self::try_from(u16::from_be_bytes([*b0, *b1]))?;
    *dw.bytes_mut() = rest;
    Ok(element)
  }
}

impl Encode<De> for CipherSuite {
  #[inline]
  fn encode(&self, ew: &mut TlsEncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_copyable_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}

impl From<CipherSuite> for u16 {
  #[inline]
  fn from(value: CipherSuite) -> Self {
    match value {
      CipherSuite::Aes128GcmSha256 => 0x1301,
      CipherSuite::Aes256GcmSha384 => 0x1302,
      CipherSuite::Chacha20Poly1305Sha256 => 0x1303,
    }
  }
}

impl TryFrom<u16> for CipherSuite {
  type Error = crate::Error;

  #[inline]
  fn try_from(value: u16) -> crate::Result<Self> {
    Ok(match value {
      0x1301 => CipherSuite::Aes128GcmSha256,
      0x1302 => CipherSuite::Aes256GcmSha384,
      0x1303 => CipherSuite::Chacha20Poly1305Sha256,
      _ => return Err(TlsError::UnsupportedCipherSuite.into()),
    })
  }
}
