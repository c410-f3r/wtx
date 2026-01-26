use crate::{
  de::{Decode, Encode},
  tls::{TlsError, de::De, decode_wrapper::DecodeWrapper, encode_wrapper::EncodeWrapper},
};

create_enum! {
  /// Refers a concrete cipher suite implementation.
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum CipherSuiteTy<u16> {
    /// TlsAes128GcmSha256
    Aes128GcmSha256 = (0x1301),
    /// TlsAes256GcmSha384
    Aes256GcmSha384 = (0x1302),
    /// TlsChacha20Poly1305Sha256
    Chacha20Poly1305Sha256 = (0x1303),
  }
}

impl CipherSuiteTy {
  #[inline]
  pub(crate) fn cipher_key_len(self) -> u8 {
    match self {
      CipherSuiteTy::Aes128GcmSha256 => 16,
      CipherSuiteTy::Aes256GcmSha384 => 32,
      CipherSuiteTy::Chacha20Poly1305Sha256 => 32,
    }
  }

  #[inline]
  pub(crate) fn hash_len(self) -> u8 {
    match self {
      CipherSuiteTy::Aes128GcmSha256 => 32,
      CipherSuiteTy::Aes256GcmSha384 => 48,
      CipherSuiteTy::Chacha20Poly1305Sha256 => 32,
    }
  }
}

impl<'de> Decode<'de, De> for CipherSuiteTy {
  #[inline]
  fn decode(dw: &mut DecodeWrapper<'de>) -> crate::Result<Self> {
    let [a, b, rest @ ..] = dw.bytes() else {
      return Err(TlsError::InvalidMaxFragmentLength.into());
    };
    *dw.bytes_mut() = rest;
    Ok(Self::try_from(u16::from_be_bytes([*a, *b]))?)
  }
}

impl Encode<De> for CipherSuiteTy {
  #[inline]
  fn encode(&self, ew: &mut EncodeWrapper<'_>) -> crate::Result<()> {
    ew.buffer().extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}
