use crate::{
  de::{Decode, Encode},
  misc::SuffixWriterMut,
  tls::{TlsError, de::De},
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

impl<'de> Decode<'de, De> for CipherSuiteTy {
  #[inline]
  fn decode(dw: &mut &'de [u8]) -> crate::Result<Self> {
    let [a, b, rest @ ..] = dw else {
      return Err(TlsError::InvalidMaxFragmentLength.into());
    };
    *dw = rest;
    Ok(Self::try_from(u16::from_be_bytes([*a, *b]))?)
  }
}

impl Encode<De> for CipherSuiteTy {
  #[inline]
  fn encode(&self, sw: &mut SuffixWriterMut<'_>) -> crate::Result<()> {
    sw.extend_from_slice(&u16::from(*self).to_be_bytes())?;
    Ok(())
  }
}
